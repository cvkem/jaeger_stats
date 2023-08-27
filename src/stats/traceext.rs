use crate::{
    processed::Trace,
    stats::{
        call_chain::{CChainEndPointCache, CChainStatsValue},
        stats_rec::StatsRec,
    },
    utils::{self, Chapter},
};
use std::{
    // error::Error,
    // fs::File,
    // io::Write,
    collections::HashMap,
    mem,
    path::Path,
};

/// Collect statistics as a string and write it to a textfile in CSV format
pub fn write_stats_to_csv_file(csv_file: &str, stats: &StatsRec) {
    //println!("Now writing the trace statistics to {csv_file}");
    let stats_csv_str = stats.to_csv_string(stats.num_files);
    if let Err(err) = utils::write_string_to_file(csv_file, stats_csv_str) {
        panic!("Writing to file '{csv_file}' failed with error: {err:?}");
    };
}

pub struct TraceExt {
    pub base_name: String,
    pub trace: Trace,
    pub stats_rec: StatsRec,
}

impl TraceExt {
    pub fn new(trace: Trace, folder: &Path, caching_processes: &[String], num_files: i32) -> Self {
        let base_name = trace.base_name(folder);

        let mut stats = StatsRec::new(caching_processes, num_files);
        stats.extend_statistics(&trace, false);

        Self {
            base_name: base_name.into_string().unwrap(),
            trace,
            stats_rec: stats,
        }
    }

    /// Translate the root_call of this trace in an endpoint-key that can be used as base for the file-name to store the call-chains for this endpoint
    pub fn get_endpoint_key(&self) -> String {
        self.trace
            .root_call
            .replace(&['/', '\\', ';', ':'][..], "_")
    }

    pub fn write_trace(&self) {
        let trace_str = format!("{:#?}", self.trace);
        let output_file = format!("{}.txt", self.base_name);
        //println!("Now writing the read Jaeger_trace to {output_file}");
        utils::write_string_to_file(&output_file, trace_str)
            .expect("Failed to write trace (.txt) to file");
    }

    pub fn fix_cchains(&mut self, cchain_cache: &mut CChainEndPointCache) {
        utils::report(
            Chapter::Details,
            format!(
                "Trace: {} does have {}",
                self.base_name,
                self.trace.missing_span_ids.len()
            ),
        );
        if let Some(expect_cc) = cchain_cache.get_cchain_key(&self.get_endpoint_key()) {
            let new_stats: HashMap<_, _> = mem::take(&mut self.stats_rec.stats)
            .into_iter()
            .map(|(key, mut stats)| {
                let (rooted, mut non_rooted): (Vec<_>, Vec<_>) = stats.call_chain
                    .into_iter()
                    .partition(|(_k2, v2)| v2.rooted);

                if !non_rooted.is_empty() {
                    let depths: Vec<_> = non_rooted.iter().map(|(_k,v)| v.depth).collect();
                    utils::report(Chapter::Details, format!("For key '{key}'  found {} non-rooted out of {} traces at depths {depths:?}", non_rooted.len(), non_rooted.len() + rooted.len()));
                }

                // fix the non-rooted paths by a rewrite of the key
                let mut fix_failed = 0;
                non_rooted.iter_mut()
                    .for_each(|(k, v)| {
                        if k.remap_callchain(expect_cc) {
                            assert!(!v.rooted);  // should be false
                            v.rooted = true;
                        } else {
                            fix_failed += 1;
                        }});
                if fix_failed > 0 {
                    utils::report(Chapter::Details, format!("Failed to fix {fix_failed} chains out of {} non-rooted chains.", non_rooted.len()));
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
                stats.call_chain = new_call_chain;
                (key, stats)
            })
            .collect();
            self.stats_rec.stats = new_stats;
        } else {
            println!("Could not find a call-chain for {}", self.trace.root_call);
        }
    }
}

pub fn build_trace_ext(
    traces: Vec<Trace>,
    folder: &Path,
    num_files: i32,
    caching_processes: &[String],
) -> Vec<TraceExt> {
    // create a traces folder
    let trace_folder = utils::extend_create_folder(folder, "Traces");

    traces
        .into_iter()
        .map(|trace| TraceExt::new(trace, &trace_folder, caching_processes, num_files))
        .collect::<Vec<_>>()
}
