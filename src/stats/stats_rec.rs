use super::{
    call_chain::{
        call_chain_key, get_call_chain, CChainEndPointCache, CChainStats, CChainStatsKey,
        CChainStatsValue,
    },
    file::StatsRecJson,
    operation_stats::OperationStats,
    proc_oper_stats::ProcOperStatsValue,
};
use crate::{
    processed::Trace,
    utils::{self, micros_to_datetime, Chapter},
};
use serde::{Deserialize, Serialize};

use chrono::NaiveDateTime;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ffi::OsString,
    mem,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

#[derive(Default, Clone)]
pub struct BasicStatsRec {
    pub num_files: i32, // i32 is more convenient for compuations than an usize
    /// number of endpoint included
    pub num_endpoints: usize,
    /// number of initial incomplete traces (before corrections)
    pub num_incomplete_traces: usize,
    // initial number of Call-chains that do not start at the root of the full trace
    pub init_num_unrooted_cc: usize,
    /// number of fixes applied
    pub num_fixes: usize,
    // Number of Call-chains that do not start at the root of the full trace after fixes based on call-chains
    pub num_unrooted_cc_after_fixes: usize,
    /// List of processes that perform caching, which is an input parameter to this analysis
    pub caching_processes: Vec<String>,
}

impl Default for Version {
    fn default() -> Self {
        Version { major: 0, minor: 2 }
    }
}

#[derive(Debug, Default, Clone)]
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
    pub num_files: i32, // i32 is more convenient for compuations than an usize
    /// number of endpoint included
    pub num_endpoints: usize,
    /// number of initial incomplete traces (before corrections)
    pub num_incomplete_traces: usize,
    // total number of call_chains
    pub num_call_chains: usize,
    // initial number of Call-chains that do not start at the root of the full trace
    pub init_num_unrooted_cc: usize,
    /// number of fixes applied
    pub num_fixes: usize,
    // Number of Call-chains that do not start at the root of the full trace after fixes based on call-chains
    pub num_unrooted_cc_after_fixes: usize,
    /// Start date-time per trace in a Naive format as the encoding in the source-files is based on Epoch-micros and does not contain time-zone information
    pub start_dt: Vec<NaiveDateTime>,
    /// End date-time for each trace
    pub end_dt: Vec<NaiveDateTime>,
    /// The duration in microseconds is added for convenience. This information is aligned with 'end_dt - start_dt'.
    pub duration_micros: Vec<i64>,
    /// The Time_to_respond_micros measures when a response is returned, as a some background computation, or writing of data might happen after this time.
    pub time_to_respond_micros: Vec<i64>,
    /// List of processes that perform caching, which is an input parameter to this analysis
    pub caching_processes: Vec<String>,
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
            num_files: srj.num_files,
            num_endpoints: srj.num_endpoints,
            num_incomplete_traces: srj.num_incomplete_traces,
            num_call_chains: srj.num_call_chains,
            init_num_unrooted_cc: srj.init_num_unrooted_cc,
            num_fixes: srj.num_fixes,
            num_unrooted_cc_after_fixes: srj.num_unrooted_cc_after_fixes,
            start_dt: srj.start_dt.into_iter().map(micros_to_datetime).collect(),
            end_dt: srj.end_dt.into_iter().map(micros_to_datetime).collect(),
            duration_micros: srj.duration_micros,
            time_to_respond_micros: srj.time_to_respond_micros,
            caching_processes: srj.caching_processes,
            stats,
        }
    }
}

impl StatsRec {
    pub fn new(mut bsr: BasicStatsRec) -> Self {
        let caching_process = mem::take(&mut bsr.caching_processes);
        let num_files = bsr.num_files;
        let num_endpoints = bsr.num_endpoints;
        let num_incomplete_traces = bsr.num_incomplete_traces;
        let init_num_unrooted_cc = bsr.init_num_unrooted_cc;
        let num_fixes = bsr.num_fixes;
        let num_unrooted_cc_after_fixes = bsr.num_unrooted_cc_after_fixes;
        StatsRec {
            caching_processes: caching_process,
            num_files,
            num_endpoints,
            num_incomplete_traces,
            init_num_unrooted_cc,
            num_fixes,
            num_unrooted_cc_after_fixes,
            ..Default::default()
        }
    }

    /// Read a StatsRecJson file and turn it into a StatsRec
    pub fn read_file(path: &OsString) -> Result<Self, Box<dyn Error>> {
        let srj = StatsRecJson::read_file(path)?;
        Ok(srj.into())
    }

    /// Calculate the contents of the call-chain-file
    pub fn call_chain_keys(&self) -> Vec<CChainStatsKey> {
        self.stats
            .values()
            .flat_map(|stat| stat.call_chain.0.keys().map(|psk| psk.clone()))
            .collect()
    }

    // /// Calculate the contents of the call-chain-file
    // pub fn call_chain_str(&self) -> String {
    //     let tmp: Vec<_> = self
    //         .stats
    //         .values()
    //         .flat_map(|stat| stat.call_chain.keys().map(|psk| psk.call_chain_key()))
    //         .collect();
    //     tmp.join("\n")
    // }

    /// extend the statistics of this StatsRec with the spans of a provided trace.
    /// The rooted_spans_only parameter determined the filtering. When set to true only spans that trace back to root are included in the analysis (parameter always false in current code)
    pub fn extend_statistics(&mut self, trace: &Trace, rooted_spans_only: bool) {
        //println!("Extend statistics for trace: {}", trace.trace_id);

        let spans = &trace.spans;

        self.trace_id.push(trace.trace_id.to_owned());
        self.root_call.push(trace.root_call.to_owned());
        self.num_spans.push(trace.spans.items.len());
        self.start_dt.push(trace.start_dt);
        self.end_dt.push(trace.end_dt);
        self.duration_micros.push(trace.duration_micros);
        self.time_to_respond_micros
            .push(trace.time_to_respond_micros);

        let mut proc_used = HashSet::new();
        // keep track of the proces/operation combinations used at least once in this process
        let mut proc_oper_used = HashSet::new();
        spans
            .items
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
                    stat.update(idx, span, &spans, &self.caching_processes, &trace.root_call);
                };

                // This is the actual insert or update based on the 'update_stats'.
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

    pub fn to_csv_string(&self) -> String {
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
                s.push(format!("num_files:; {}", self.num_files));
                s.push(format!("num_endpoints:; {}", self.num_endpoints));
                s.push(format!(
                    "num_incomplete_traces:; {}",
                    self.num_incomplete_traces
                ));
                s.push(format!(
                    "init_num_unrooted_cc:; {}",
                    self.init_num_unrooted_cc
                ));
                s.push(format!("num_fixes:; {}", self.num_fixes));
                s.push(format!(
                    "num_unrooted_cc_after_fixes:; {}",
                    self.num_unrooted_cc_after_fixes
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
                let line = meth_stat.report_stats_line(k, method, num_traces, self.num_files);
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
                    .0
                    .iter()
                    .map(|(ps_key, cchain_stats)| (ps_key, key.to_owned(), cchain_stats))
            })
            .collect::<Vec<_>>();
        ps_data.sort_by(|a, b| a.0.cmp(b.0));
        ps_data.into_iter().for_each(|(ps_key, key, cchain_stats)| {
            s.push(cchain_stats.report_stats_line(&key, ps_key, num_traces, self.num_files))
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
                    .0
                    .keys()
                    .map(|ps_key| call_chain_key(&ps_key.call_chain, "", ps_key.is_leaf))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// internal function
    pub fn count_call_chains(&self) -> (usize, usize) {
        let total_cc = self
            .stats
            .values()
            .map(|st| st.call_chain.0.keys().count())
            .count();

        let unrooted_cc = self
            .stats
            .values()
            .map(|st| st.call_chain.0.values().filter(|ccv| !ccv.rooted).count())
            .count();
        (total_cc, unrooted_cc)
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

    pub fn fix_call_chain(&mut self, cchain_cache: &mut CChainEndPointCache) -> usize {
        let mut num_fixes = 0;

        //        if let Some(expect_cc) = cchain_cache.get_cchain_key(&self.get_endpoint_key()) {
        let new_stats: HashMap<_, _> = mem::take(&mut self.stats)
            .into_iter()
            .map(|(key, mut stats)| {
                    let (rooted, mut non_rooted): (Vec<_>, Vec<_>) = stats.call_chain.0
                        .into_iter()
                        .partition(|(_k2, v2)| v2.rooted);

                    if !non_rooted.is_empty() {
                        let depths: Vec<_> = non_rooted.iter().map(|(_k,v)| v.depth).collect();
                        utils::report(Chapter::Details, format!("For key '{key}'  found {} non-rooted out of {} traces with call-chain depths {depths:?}", non_rooted.len(), non_rooted.len() + rooted.len()));
                    }

                    // fix the non-rooted paths by a rewrite of the key
                    let mut fix_failed = 0;
                    non_rooted.iter_mut()
                        .for_each(|(k, v)| {
                            if let Some(expect_cc) = v
                                .expect_root
                                .get_frequent_end_point()
                                .and_then(|end_point| cchain_cache.get_cchain_key(&CChainEndPointCache::str_to_cache_key(&end_point)))
                            {
                                if k.remap_callchain(expect_cc) {
                                    assert!(!v.rooted);  // should be false
                                    num_fixes += 1;
                                    v.rooted = true;
                                } else {
                                    fix_failed += 1;
                                }
                            } else {
                                println!("Failed to find call-chain");
                                fix_failed += 1;
                            }
                    });
                    if fix_failed > 0 {
                        utils::report(Chapter::Details, format!("Failed to fix {fix_failed} chains out of {} non-rooted chains. ({num_fixes} fixes applied succesful)", non_rooted.len()));
                    }

                    let new_call_chain = rooted.into_iter()
                        .chain(non_rooted.into_iter())
                        .fold(HashMap::new(), |mut cc, (k, mut v_new)| {
                            cc.entry(k)
                                .and_modify(|v_curr: &mut CChainStatsValue| {
                                    v_curr.count += v_new.count;
                                    v_curr.duration_micros.append(&mut v_new.duration_micros);
                                })
                                .or_insert(v_new);
                            cc
                        });
                    stats.call_chain = CChainStats( new_call_chain );
                    (key, stats)
            })
            .collect();
        self.stats = new_stats;

        num_fixes
    }
}

/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn chained_stats(trace: &Trace) -> HashMap<String, u32> {
    let spans = &trace.spans;

    let mut stats = HashMap::new();
    spans.items.iter().enumerate().for_each(|(idx, span)| {
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
