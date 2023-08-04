use crate::{
    stats::call_chain::{
        caching_process_label, call_chain_key, 
        Call, CallChain,
        CChainStats, CChainStatsKey, CChainStatsValue},
    processed::{Spans, Trace},
};
use super::method_stats::{MethodStats, MethodStatsValue};

use chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

//type ProcessKey = String;    // does not deliver any additional type-safety

#[derive(Debug, Default)]
pub struct Stats {
    pub num_received_calls: usize, // inbound calls to this process
    pub num_outbound_calls: usize, // outbound calls to other processes
    pub num_unknown_calls: usize,
    pub method: MethodStats,
    //    method_cache_suffix: HashMap<String, usize>,  // methods in a cache-chain have a suffix.
    pub call_chain: CChainStats,
}

impl Stats {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct StatsRec {
    pub trace_id: Vec<String>,
    pub root_call: Vec<String>,
    pub num_spans: Vec<usize>,
    pub num_files: i32, // the number of files is needed when computing the rate of requests.
    pub start_dt: Vec<DateTime<Utc>>,
    pub end_dt: Vec<DateTime<Utc>>,
    pub duration_micros: Vec<i64>,
    pub time_to_respond_micros: Vec<i64>,
    pub caching_process: Vec<String>,
    pub stats: HashMap<String, Stats>, // hashmap base on the leaf process (as that is the initial level of reporting)
}

impl StatsRec {
    pub fn new(caching_process: &Vec<String>, num_files: i32) -> Self {
        let caching_process = caching_process.clone();
        StatsRec {
            caching_process,
            num_files,
            ..Default::default()
        }
    }

    /// Calculate the contents of the call-chain-file
    pub fn call_chain_str(&self) -> String {
        let tmp: Vec<_> = self
            .stats
            .values()
            .flat_map(|stat| stat.call_chain.keys().map(|psk| psk.call_chain_key()))
            .collect();
        tmp.join("\n")
    }

    /// extend the statistics of this StatsRec with the spans of a provided trace.
    /// The rooted_spans_only parameter determined the filtering. When set to true only spans that trace back to root are included in the analysis (parameter always false in current code)
    pub fn extend_statistics(&mut self, trace: &Trace, rooted_spans_only: bool) {
        //println!("Extend statistics for trace: {}", trace.trace_id);

        let spans = &trace.spans;

        self.trace_id.push(trace.trace_id.to_owned());
        self.root_call.push(trace.root_call.to_owned());
        self.num_spans.push(trace.spans.len());
        self.start_dt.push(trace.start_dt);
        self.end_dt.push(trace.end_dt);
        self.duration_micros.push(trace.duration_micros);
        self.time_to_respond_micros
            .push(trace.time_to_respond_micros);

        spans
            .iter()
            .enumerate()
            .filter(|(_, span)| {
                // determines which traces to include
                if rooted_spans_only {
                    span.rooted
                } else {
                    true
                }
            })
            .for_each(|(idx, span)| {
                let proc = span.get_process_str().to_owned();
                let method = &span.operation_name;
                let update_stat = |stat: &mut Stats| {
                    match &span.span_kind {
                        Some(kind) => match &kind[..] {
                            "server" => stat.num_received_calls += 1,
                            "client" => stat.num_outbound_calls += 1,
                            _ => stat.num_unknown_calls += 1,
                        },
                        None => stat.num_unknown_calls += 1,
                    }

                    let duration_micros = span.duration_micros;
                    let start_dt_micros = span.start_dt.timestamp_micros();
                    // add a count per method
                    stat.method
                        .0
                        .entry(method.to_owned())
                        .and_modify(|meth_stat| {
                            meth_stat.count += 1;
                            meth_stat.start_dt_micros.push(start_dt_micros);
                            meth_stat.duration_micros.push(duration_micros);
                        })
                        .or_insert(MethodStatsValue::new(duration_micros, start_dt_micros));

                    // // add a count per method_including-cached
                    let call_chain = get_call_chain(idx, &spans);
                    let caching_process = caching_process_label(&self.caching_process, &call_chain);

                    // add call-chain stats
                    let depth = call_chain.len();
                    let looped = get_duplicates(&call_chain);
                    let is_leaf = span.is_leaf;
                    let rooted = span.rooted;

                    let ps_key = CChainStatsKey {
                        call_chain,
                        caching_process,
                        is_leaf,
                    };
                    stat.call_chain
                        .entry(ps_key)
                        .and_modify(|ps| {
                            ps.count += 1;
                            ps.start_dt_micros.push(start_dt_micros);
                            ps.duration_micros.push(duration_micros);
                        })
                        .or_insert_with(|| {
                            let dms: Box<[_]> = Box::new([duration_micros]);
                            let duration_micros = dms.into_vec();
                            let start_dt_micros = [start_dt_micros].to_vec();
                            CChainStatsValue {
                                count: 1,
                                depth,
                                duration_micros,
                                start_dt_micros,
                                looped,
                                rooted,
                            }
                        });
                };

                self.stats
                    .entry(proc)
                    .and_modify(update_stat)
                    .or_insert_with(|| {
                        let mut stat = Stats::new();
                        update_stat(&mut stat);
                        stat
                    });
            });
    }

    pub fn to_csv_string(&self, num_files: i32) -> String {
        let mut s = Vec::new();
        let num_traces: i64 = self.trace_id.len().try_into().unwrap();

        match num_traces {
            0 => panic!("No data in Stats"),
            1 => {
                s.push(format!("trace_id:; {}", self.trace_id[0]));
                s.push(format!("root_call:; {}", self.root_call[0]));
                s.push(format!("num_spans:; {}", self.num_spans[0]));
                s.push(format!("start_dt; {:?}", self.start_dt[0]));
                s.push(format!("end_dt:; {:?}", self.end_dt[0]));
                s.push(format!("duration_micros:; {}", self.duration_micros[0]));
                s.push(format!(
                    "time_to_respond_micros:; {}",
                    self.time_to_respond_micros[0]
                ));
            }
            n => {
                s.push(format!("trace_ids:; {:?}", self.trace_id));
                s.push(format!("num_traces:; {num_traces}"));
                s.push(format!(
                    "MIN(num_spans):; {:?}",
                    self.num_spans.iter().min().unwrap()
                ));
                s.push(format!(
                    "AVG(num_spans):; {:?}",
                    self.num_spans.iter().sum::<usize>() as i64 / n
                ));
                s.push(format!(
                    "MAX(num_spans):; {:?}",
                    self.num_spans.iter().max().unwrap()
                ));
                s.push(format!(
                    "root_call_stats:; {}",
                    root_call_stats(&self.root_call)
                ));
                s.push(format!(
                    "root_calls:; {:?}",
                    root_call_list(&self.trace_id, &self.root_call)
                ));
                s.push(format!("start_dt; {:?}", self.start_dt));
                s.push(format!("end_dt:; {:?}", self.end_dt));
                s.push(format!(
                    "MIN(duration_micros):; {:?}",
                    self.duration_micros.iter().min().unwrap()
                ));
                s.push(format!(
                    "AVG(duration_micros):; {:?}",
                    self.duration_micros.iter().sum::<i64>() / n
                ));
                s.push(format!(
                    "MAX(duration_micros):; {:?}",
                    self.duration_micros.iter().max().unwrap()
                ));
                s.push(format!("duration_micros:; {:?}", self.duration_micros));
                s.push(format!(
                    "MIN(time_to_respond_micros):; {:?}",
                    self.time_to_respond_micros.iter().min().unwrap()
                ));
                s.push(format!(
                    "AVG(time_to_respond):; {:?}",
                    self.time_to_respond_micros.iter().sum::<i64>() / n
                ));
                s.push(format!(
                    "MAX(time_to_respond_micros):; {:?}",
                    self.time_to_respond_micros.iter().max().unwrap()
                ));
                s.push(format!(
                    "time_to_respond_micros:; {:?}",
                    self.time_to_respond_micros
                ));
            }
        }
        s.push("\n".to_owned());

        let mut data: Vec<_> = self.stats.iter().collect();
        data.sort_by(|a, b| a.0.cmp(&b.0));

        s.push("Process; Num_received_calls; Num_outbound_calls; Num_unknown_calls; Perc_received_calls; Perc_outbound_calls; Perc_unknown_calls".to_owned());
        data.iter().for_each(|(k, stat)| {
            let freq_rc = stat.num_received_calls as f64 / num_traces as f64;
            let freq_oc = stat.num_outbound_calls as f64 / num_traces as f64;
            let freq_uc = stat.num_outbound_calls as f64 / num_traces as f64;
            let line = format!(
                "{k}; {}; {}; {}; {}; {}; {}",
                stat.num_received_calls,
                stat.num_outbound_calls,
                stat.num_unknown_calls,
                format_float(freq_rc),
                format_float(freq_oc),
                format_float(freq_uc)
            );
            s.push(line);
        });
        s.push("\n".to_owned());

        let num_traces = num_traces as f64;
        s.push("Process; Count; Min_millis; Avg_millis; Max_millis; Percentage; Rate; Expect_duration;".to_owned());
        data.iter().for_each(|(k, stat)| {
            stat.method.0.iter().for_each(|(method, meth_stat)| {
                let line = meth_stat.report_stats_line(k, method, num_traces, num_files);
                s.push(line);
            })
        });
        s.push("\n".to_owned());

        s.push("#The unique key of the next table is 'Call_Chain' (which includes full path and the leaf-marker). So the Process column contains duplicates".to_owned());

        s.push("Call_chain; cc_hash; End_point; Process/operation; Is_leaf; Depth; Count; Looped; Revisit; Caching_proces; Min_millis; Avg_millis; Max_millis; Percentage; Rate; expect_duration; expect_contribution;".to_owned());

        // reorder data based on the full call-chain
        //  key is the ProcessKey and ps_key is the PathStatsKey (a.o. call-chain)
        let mut ps_data = data
            .into_iter()
            .flat_map(|(key, stat)| {
                stat.call_chain
                    .iter()
                    .map(|(ps_key, cchain_stats)| (ps_key, key.to_owned(), cchain_stats))
            })
            .collect::<Vec<_>>();
        ps_data.sort_by(|a, b| a.0.cmp(&b.0));
        ps_data.into_iter().for_each(|(ps_key, key, cchain_stats)| {
            s.push(cchain_stats.report_stats_line(&key, ps_key, num_traces, num_files))
        });
        s.push("\n".to_owned());

        s.join("\n")
    }

    /// internal function
    fn call_chain_list(&self) -> Vec<String> {
        self.stats
            .values()
            .flat_map(|st| {
                st.call_chain
                    .keys()
                    .map(|ps_key| call_chain_key(&ps_key.call_chain, &"", ps_key.is_leaf))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// resturns a hashset containing all call-chains (string-keys)
    pub fn call_chain_set(&self) -> HashSet<String> {
        let cc_keys = self.call_chain_list();
        HashSet::from_iter(cc_keys.into_iter())
    }

    /// returns a hashset containing all call-chains (string-keys)
    pub fn call_chain_sorted(&self) -> Vec<String> {
        let mut cc_keys = self.call_chain_list();
        cc_keys.sort();
        cc_keys
    }
}

/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn basic_stats(trace: &Trace) -> HashMap<String, u32> {
    let spans = &trace.spans;

    let mut stats = HashMap::new();
    spans.iter().for_each(|span| {
        let proc = span.get_process_str();
        let proc_method = format!("{}/{}", proc, span.operation_name);
        stats
            .entry(proc_method)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    });
    stats
}

static COMMA_FLOAT: Mutex<bool> = Mutex::new(false);

pub fn set_comma_float(val: bool) {
    let mut guard = COMMA_FLOAT.lock().unwrap();
    *guard = val
}

/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
pub fn format_float(val: f64) -> String {
    let s = format!("{}", val);
    if *COMMA_FLOAT.lock().unwrap() {
        s.replace('.', ",")
    } else {
        s
    }
}

/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
pub fn format_float_opt(val: Option<f64>) -> String {
    match val {
        Some(v) => format_float(v),
        None => "--".to_owned(),
    }
}

/// get_call_chain returns the full call_chain from top to bottom showing process and called method
/// this function does a recursive trace back to identify all parent-links:
/// TODO: move to  module that is related to CallChain. It now is difficult to find.in the code-base
fn get_call_chain(idx: usize, spans: &Spans) -> CallChain {
    let span = &spans[idx];
    // find the root and allocate vector
    let mut call_chain = match span.parent {
        None => Vec::new(),
        Some(idx) => get_call_chain(idx, spans),
    };
    // and push all proces names starting from the root
    let process = span.get_process_str().to_owned();
    let method = span.operation_name.to_owned();
    let call_direction = span.span_kind.as_ref().into();
    call_chain.push(Call {
        process,
        method,
        call_direction,
    });
    call_chain
}

/// get all values that appear more than once in the list of strings, while being none-adjacent.
fn get_duplicates(names: &CallChain) -> Vec<String> {
    let mut duplicates = Vec::new();
    for idx in 0..names.len() {
        let proc = &names[idx].process;
        let mut j = 0;
        loop {
            if j >= duplicates.len() {
                break;
            }
            if duplicates[j] == *proc {
                break;
            }
            j += 1;
        }
        if j < duplicates.len() {
            continue;
        }
        //  nme does not exist in duplicates yet, so find it in names
        let mut j = idx + 2; // Step by 2 as we want to prevent matching sub-sequent GET calls
        loop {
            if j >= names.len() || names[j].process == *proc {
                break;
            }
            j += 1;
        }
        if j < names.len() {
            duplicates.push(proc.to_owned());
        }
    }
    duplicates
}

/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn chained_stats(trace: &Trace) -> HashMap<String, u32> {
    let spans = &trace.spans;

    let mut stats = HashMap::new();
    spans.iter().enumerate().for_each(|(idx, span)| {
        let proc = span.get_process_str();
        let parents_str = get_call_chain(idx, spans)
            .into_iter()
            .fold(String::new(), |a, b| a + &b.process + &b.method + " | ");
        let proc_method = format!("{parents_str}{proc}/{}", span.operation_name);
        stats
            .entry(proc_method)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    });
    stats
}

/// root_call_stats return a list of root_calls and their count.
fn root_call_stats(root_calls: &Vec<String>) -> String {
    let mut stats = HashMap::new();
    root_calls.iter().for_each(|call| {
        stats
            .entry(call)
            .and_modify(|counter: &mut i32| *counter += 1)
            .or_insert(1);
    });
    let mut data: Vec<_> = stats.iter().collect();
    data.sort_by(|a, b| b.1.cmp(&a.1));
    format!("{data:?}")
}

fn root_call_list(trace_ids: &Vec<String>, root_calls: &Vec<String>) -> String {
    let labelled: Vec<_> = trace_ids
        .iter()
        .zip(root_calls.iter())
        .map(|(tr, rc)| format!("{tr} -> {rc}"))
        .collect();
    labelled.join(",   ")
}
