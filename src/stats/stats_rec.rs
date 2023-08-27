use super::{
    call_chain::{call_chain_key, get_call_chain, CChainStatsValue},
    file::StatsRecJson,
    operation_stats::OperationStats,
    proc_oper_stats::ProcOperStatsValue,
};
use crate::{processed::Trace, utils::micros_to_datetime};
use serde::{Deserialize, Serialize};

use chrono::NaiveDateTime;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ffi::OsString,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl Default for Version {
    fn default() -> Self {
        Version { major: 0, minor: 2 }
    }
}

#[derive(Debug, Default)]
pub struct StatsRec {
    /// version numbering to handle diversity of analyzed data-sets.
    /// Currently we do not have the code that handles or checks versions. So only present for post-mortem analysis or future usage.
    pub version: Version,
    /// Ordered list of all the trace_ids present in this
    pub trace_id: Vec<String>,
    /// The root-method (initial call) of each of the traces
    pub root_call: Vec<String>,
    /// The total number of spans per trace
    pub num_spans: Vec<usize>,
    /// The number of input-files used to collect this set of traces. This number is eeded when computing the rate of requests as we need to correct for possible gaps between files
    pub num_files: i32,
    /// Start date-time per trace in a Naive format as the encoding in the source-files is based on Epoch-micros and does not contain time-zone information
    pub start_dt: Vec<NaiveDateTime>,
    /// End date-time for each trace
    pub end_dt: Vec<NaiveDateTime>,
    /// The duration in microseconds is added for convenience. This information is aligned with 'end_dt - start_dt'.
    pub duration_micros: Vec<i64>,
    /// The Time_to_respond_micros measures when a response is returned, as a some background computation, or writing of data might happen after this time.
    pub time_to_respond_micros: Vec<i64>,
    /// List of processes that perform caching, which is an input parameter to this analysis
    pub caching_process: Vec<String>,
    /// Statistis per leaf-process (end-point of the chain of processes)
    pub stats: HashMap<String, OperationStats>, // hashmap based on the leaf process (as that is the initial level of reporting)
}

impl From<StatsRecJson> for StatsRec {
    fn from(srj: StatsRecJson) -> Self {
        let stats: HashMap<String, OperationStats> =
            srj.stats.into_iter().map(|(k, v)| (k, v.into())).collect();
        Self {
            version: srj.version,
            trace_id: srj.trace_id,
            root_call: srj.root_call,
            num_spans: srj.num_spans,
            num_files: srj.num_files, // Was optional for backward compatibiliy
            start_dt: srj.start_dt.into_iter().map(micros_to_datetime).collect(),
            end_dt: srj.end_dt.into_iter().map(micros_to_datetime).collect(),
            duration_micros: srj.duration_micros,
            time_to_respond_micros: srj.time_to_respond_micros,
            caching_process: srj.caching_process,
            stats,
        }
    }
}

impl StatsRec {
    pub fn new(caching_process: &[String], num_files: i32) -> Self {
        let caching_process = caching_process.to_owned();
        StatsRec {
            caching_process,
            num_files,
            ..Default::default()
        }
    }

    /// Read a StatsRecJson file and turn it into a StatsRec
    pub fn read_file(path: &OsString) -> Result<Self, Box<dyn Error>> {
        let srj = StatsRecJson::read_file(path)?;
        Ok(srj.into())
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

        let mut proc_used = HashSet::new();
        // keep track of the proces/operation combinations used at least once in this process
        let mut proc_oper_used = HashSet::new();
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
                let proc = span.get_process_str();

                // keep track of the proces/operation (via &str references)
                let _ = proc_used.insert(proc);
                let _ = proc_oper_used.insert((proc, &span.operation_name));
                let proc = proc.to_owned();

                let update_stat = |stat: &mut OperationStats| {
                    stat.update(idx, span, spans, &self.caching_process);
                };

                // This is the actual insert or update baed on the 'update_stats'.
                self.stats
                    .entry(proc)
                    .and_modify(update_stat)
                    .or_insert_with(|| {
                        let mut stat = OperationStats::new();
                        update_stat(&mut stat);
                        stat
                    });
            });

        // Update num_traces such that each process is uniquely counted.
        proc_used.into_iter().for_each(|proc| {
            self.stats
                .entry(proc.to_owned())
                .and_modify(|st| st.num_traces += 1);
        });
        proc_oper_used.into_iter().for_each(|(proc, oper)| {
            self.stats.entry(proc.to_owned()).and_modify(|st| {
                st.operation
                    .0
                    .entry(oper.to_owned())
                    .and_modify(|oper| oper.num_traces += 1);
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
        data.sort_by(|a, b| a.0.cmp(b.0));

        s.push(OperationStats::report_stats_line_header_str().to_owned());
        data.iter()
            .for_each(|(k, stat)| s.push(stat.report_stats_line(k, num_traces as f64)));
        s.push("\n".to_owned());

        let num_traces = num_traces as f64;
        s.push(ProcOperStatsValue::report_stats_line_header_str().to_owned());
        data.iter().for_each(|(k, stat)| {
            stat.operation.0.iter().for_each(|(method, meth_stat)| {
                let line = meth_stat.report_stats_line(k, method, num_traces, num_files);
                s.push(line);
            })
        });
        s.push("\n".to_owned());

        s.push("#The unique key of the next table is 'Call_Chain' (which includes full path and the leaf-marker). So the Process column contains duplicates".to_owned());

        s.push(CChainStatsValue::report_stats_line_header_str().to_owned());

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
        ps_data.sort_by(|a, b| a.0.cmp(b.0));
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
                    .map(|ps_key| call_chain_key(&ps_key.call_chain, "", ps_key.is_leaf))
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
        cc_keys.sort_unstable();
        cc_keys
    }
}

// /// Compute basic call statistics, which only looks at functions/operations and does not include the call path
// pub fn basic_stats(trace: &Trace) -> HashMap<String, u32> {
//     let spans = &trace.spans;

//     let mut stats = HashMap::new();
//     spans.iter().for_each(|span| {
//         let proc = span.get_process_str();
//         let proc_method = format!("{}/{}", proc, span.operation_name);
//         stats
//             .entry(proc_method)
//             .and_modify(|counter| *counter += 1)
//             .or_insert(1);
//     });
//     stats
// }

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
fn root_call_stats(root_calls: &[String]) -> String {
    let mut stats = HashMap::new();
    root_calls.iter().for_each(|call| {
        stats
            .entry(call)
            .and_modify(|counter: &mut i32| *counter += 1)
            .or_insert(1);
    });
    let mut data: Vec<_> = stats.iter().collect();
    data.sort_by(|a, b| b.1.cmp(a.1));
    format!("{data:?}")
}

fn root_call_list(trace_ids: &[String], root_calls: &[String]) -> String {
    let labelled: Vec<_> = trace_ids
        .iter()
        .zip(root_calls.iter())
        .map(|(tr, rc)| format!("{tr} -> {rc}"))
        .collect();
    labelled.join(",   ")
}
