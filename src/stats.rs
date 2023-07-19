use std::{
    collections::{HashMap, HashSet},
    sync::Mutex};
use crate::{
    cchain_stats::{CChainStatsKey, CChainStatsValue, CChainStats},
    method_stats::{MethodStats, MethodStatsValue},
    call_chain::{Call, CallChain, caching_process_label, call_chain_key},
    Trace,
    span::Spans};
use chrono::{
    DateTime,
    Utc};



type ProcessKey = String;

#[derive(Debug, Default)]
pub struct Stats {
    pub num_received_calls: usize,  // inbound calls to this process
    pub num_outbound_calls: usize,  // outbound calls to other processes
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
    pub start_dt: Vec<DateTime<Utc>>,
    pub end_dt: Vec<DateTime<Utc>>,
    pub duration_micros: Vec<u64>,
    pub time_to_respond_micros: Vec<u64>,
    pub caching_process: Vec<String>,
    pub stats: HashMap<String, Stats>
}

impl StatsRec {

    pub fn new(caching_process: &Vec<String>) -> Self {
        let caching_process = caching_process.clone();
        StatsRec{
            caching_process,
            ..Default::default()}
    }

    /// Calcullate the contents of the call-chain-file
    pub fn call_chain_str(&self) -> String {
        let tmp: Vec<_> = self.stats
            .values()
            .flat_map(|stat| {
                stat.call_chain
                    .keys()
                    .map(|psk| psk.call_chain_key())
            })
//            .intersperse("\n")
            .collect();
        tmp.join("\n")
    }

    pub fn extend_statistics(&mut self, trace: &Trace, rooted_spans: bool) {

        //println!("Extend statistics for trace: {}", trace.trace_id);

        let spans = &trace.spans;

        self.trace_id.push(trace.trace_id.to_owned());
        self.root_call.push(trace.root_call.to_owned());
        self.num_spans.push(trace.spans.len());
        self.start_dt.push(trace.start_dt);
        self.end_dt.push(trace.end_dt);
        self.duration_micros.push(trace.duration_micros);
        self.time_to_respond_micros.push(trace.time_to_respond_micros);

        spans
            .iter()
            .enumerate()
            .filter(|(idx, span)| {
                if rooted_spans {
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
                            _ => stat.num_unknown_calls += 1

                        },
                        None => stat.num_unknown_calls += 1
                    }

                    let duration_micros = span.duration_micros;
                    // add a count per method
                    stat.method
                        .entry(method.to_owned())
                        .and_modify(|meth_stat| {
                            meth_stat.count += 1;
                            meth_stat.duration_micros.push(duration_micros);})
                        .or_insert(MethodStatsValue::new(duration_micros));

                    // // add a count per method_including-cached
                    let call_chain = get_call_chain(idx, &spans);
                    let caching_process = caching_process_label(&self.caching_process, &call_chain);

                    // add call-chain stats
                    let depth = call_chain.len();
                    let looped = get_duplicates(&call_chain);
                    let is_leaf = span.is_leaf;
                    let rooted = span.rooted;

                    let ps_key = CChainStatsKey{call_chain, caching_process, is_leaf};
                    stat.call_chain
                        .entry(ps_key)
                        .and_modify(|ps| {
                            ps.count += 1;
                            ps.duration_micros.push(duration_micros);})
                        .or_insert_with(|| {
                            let dms: Box<[_]> = Box::new([duration_micros]);
                            let duration_micros = dms.into_vec();
                            CChainStatsValue{count: 1, depth, duration_micros, looped, rooted}
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


    pub fn to_csv_string(&self) -> String {
        let mut s = Vec::new();
        let num_traces = self.trace_id.len() as u64;

        match num_traces {
            0 => panic!("No data in Stats"),
            1 => {
                s.push(format!("trace_id:; {}", self.trace_id[0]));
                s.push(format!("root_call:; {}", self.root_call[0]));
                s.push(format!("num_spans:; {}", self.num_spans[0]));
                s.push(format!("start_dt; {:?}", self.start_dt[0]));
                s.push(format!("end_dt:; {:?}", self.end_dt[0]));
                s.push(format!("duration_micros:; {}", self.duration_micros[0]));
                s.push(format!("time_to_respond_micros:; {}", self.time_to_respond_micros[0]));
        
            },
            n => {
                s.push(format!("trace_ids:; {:?}", self.trace_id));
                s.push(format!("num_traces:; {num_traces}"));
                s.push(format!("MIN(num_spans):; {:?}", self.num_spans.iter().min().unwrap()));
                s.push(format!("AVG(num_spans):; {:?}", self.num_spans.iter().sum::<usize>() as u64/n));
                s.push(format!("MAX(num_spans):; {:?}", self.num_spans.iter().max().unwrap()));
                s.push(format!("root_call_stats:; {}", root_call_stats(&self.root_call)));
                s.push(format!("root_calls:; {:?}", root_call_list(&self.trace_id, &self.root_call)));
                s.push(format!("start_dt; {:?}", self.start_dt));
                s.push(format!("end_dt:; {:?}", self.end_dt));
                s.push(format!("MIN(duration_micros):; {:?}", self.duration_micros.iter().min().unwrap()));
                s.push(format!("AVG(duration_micros):; {:?}", self.duration_micros.iter().sum::<u64>()/n));
                s.push(format!("MAX(duration_micros):; {:?}", self.duration_micros.iter().max().unwrap()));
                s.push(format!("duration_micros:; {:?}", self.duration_micros));
                s.push(format!("MIN(time_to_respond_micros):; {:?}", self.time_to_respond_micros.iter().min().unwrap()));
                s.push(format!("AVG(time_to_respond):; {:?}", self.time_to_respond_micros.iter().sum::<u64>()/n));
                s.push(format!("MAX(time_to_respond_micros):; {:?}", self.time_to_respond_micros.iter().max().unwrap()));
                s.push(format!("time_to_respond_micros:; {:?}", self.time_to_respond_micros));        
            }
        }
        s.push("\n".to_owned());

        let mut data: Vec<_> = self.stats.iter().collect();
        data.sort_by(|a,b| { a.0.cmp(&b.0)});

        s.push("Process; Num_received_calls; Num_outbound_calls; Num_unknown_calls; Freq_received_calls; Freq_outbound_calls; Freq_unknown_calls".to_owned());
        data.iter()
            .for_each(|(k, stat)| {
                let freq_rc = stat.num_received_calls as f64/ num_traces as f64;
                let freq_oc = stat.num_outbound_calls as f64/ num_traces as f64;
                let freq_uc = stat.num_outbound_calls as f64/ num_traces as f64;
                let line = format!("{k}; {}; {}; {}; {}; {}; {}", 
                    stat.num_received_calls, stat.num_outbound_calls, stat.num_unknown_calls,
                    format_float(freq_rc), format_float(freq_oc), format_float(freq_uc));
                s.push(line);
            });
        s.push("\n".to_owned());

        let num_traces = num_traces as f64; 
        s.push("Process; Count; min_millis; avg_millis; max_millis; freq.; expect_duration;".to_owned());
        data.iter()
            .for_each(|(k, stat)| {
                stat.method
                    .iter()
                    .for_each(|(method, meth_stat)| {
                        let line = meth_stat.report_stats_line(k, method, num_traces);
                        s.push(line);
                            })
            });
        s.push("\n".to_owned());

        s.push("#The unique key of the next table is 'Call_Chain' (which includes full path and the leaf-marker). So the Process column contains duplicates".to_owned());

        s.push("Process; Is_leaf; Depth; Count; Looped; Revisit; Call_chain; min_millis; avg_millis; max_millis; freq.; expect_duration; expect_contribution;".to_owned());

        // reorder data based on the full call-chain
        //  key is the ProcessKey and ps_key is the PathStatsKey (a.o. call-chain)
        let mut ps_data = data
            .into_iter()
            .flat_map(|(key, stat)| {
                stat.call_chain
                    .iter()
                    .map(|(ps_key, cchain_stats)| {
                        (ps_key, key.to_owned(), cchain_stats)
                    })
            })
            .collect::<Vec<_>>();
        ps_data.sort_by(|a,b| { a.0.cmp(&b.0)});
        ps_data
            .into_iter()
            .for_each(|(ps_key, key, cchain_stats)| s.push(cchain_stats.report_stats_line(&key, ps_key, num_traces)));
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
                    .map(|ps_key|{
                        call_chain_key(&ps_key.call_chain, &"", ps_key.is_leaf)
                    })
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
    spans
        .iter()
        .for_each(|span| {
            let proc = span.get_process_str();
            let proc_method = format!("{}/{}", proc, span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}


static COMMA_FLOAT: Mutex<bool> = Mutex::new(false);

pub fn set_comma_float(val: bool) {
    let mut  guard = COMMA_FLOAT.lock().unwrap();
    *guard = val
}


/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
pub fn format_float(val: f64) -> String {
    let s = format!("{}", val);
    if *COMMA_FLOAT.lock().unwrap() {
        s.replace('.',",")
    } else {
        s
    }
}


/// parent_call_chain returns the full call_chain from top to bottom showing process and called method
fn get_call_chain(idx: usize, spans: &Spans) -> CallChain {
    let span = &spans[idx];
    // find the root and allocate vector
    let mut call_chain = match span.parent {
         None =>  Vec::new(),
         Some(idx) => get_call_chain(idx, spans)
        };
    // and push all proces names starting from the root
    let process = span.get_process_str().to_owned();
    let method = span.operation_name.to_owned();
    call_chain.push( Call{process, method} );
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
            if j>= names.len() || names[j].process == *proc {
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
    spans
        .iter()
        .enumerate()
        .for_each(|(idx, span)| {
            let proc = span.get_process_str();
            let parents_str = get_call_chain(idx, spans)
                .into_iter()
                .fold(String::new(), |a, b| a + &b.process + &b.method +" | ");
            let proc_method = format!("{parents_str}{proc}/{}", span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}


/// root_call_stats return a list of root_calls and their count.
fn root_call_stats(root_calls: &Vec<String>) -> String {
    let mut stats = HashMap::new();
    root_calls
        .iter()
        .for_each(|call| {
            stats
                .entry(call)
                .and_modify(|counter: &mut i32| *counter += 1)
                .or_insert(1);
        });
    let mut data: Vec<_> = stats.iter().collect();
    data.sort_by(|a,b| { b.1.cmp(&a.1)});
    format!("{data:?}")
}

fn root_call_list(trace_ids: &Vec<String>, root_calls: &Vec<String>) -> String {
    let labelled: Vec<_> = trace_ids
        .iter()
        .zip(root_calls.iter())
        .map(|(tr, rc)| format!("{tr} -> {rc}"))
        .collect();
    labelled
        .join(",   ")
}