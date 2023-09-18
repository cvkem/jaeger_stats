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

use super::{call_chain::CChainStats, stats_rec::BasicStatsRec};

/// Collect statistics as a string and write it to a textfile in CSV format
pub fn write_stats_to_csv_file(csv_file: &str, stats: &StatsRec) {
    //println!("Now writing the trace statistics to {csv_file}");
    let stats_csv_str = stats.to_csv_string();
    if let Err(err) = utils::write_string_to_file(csv_file, stats_csv_str) {
        panic!("Writing to file '{csv_file}' failed with error: {err:?}");
    };
}

pub struct TraceExt {
    pub base_name: String,
    pub trace: Trace,
    //REMOVE
    //    pub stats_rec: StatsRec,
}

impl TraceExt {
    pub fn new(trace: Trace, folder: &Path, mut bsr: BasicStatsRec) -> Self {
        let base_name = trace.base_name(folder);

        bsr.num_files = 1;
        bsr.init_num_incomplete_traces = if trace.missing_span_ids.is_empty() {
            0
        } else {
            1
        };

        let mut stats = StatsRec::new(bsr); // collects statistics over single trace, so 1 file
        stats.extend_statistics(&trace, false);

        Self {
            base_name: base_name.into_string().unwrap(),
            trace,
            //           stats_rec: stats,
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

    //REMOVE
    //    /// fix_cchains tries to repair the call-chains by looking up call-chains observed in the past from complete traces.
    //    /// Please note that this function only fixes the call-chain information, and does NOT repair the underlying trace (yet).
    //   /// TODO: correct for root-chains and non-rooted chains as we should have that infomation
    // pub fn fix_cchains(&mut self, cchain_cache: &mut CChainEndPointCache) -> usize {
    //     let mut num_fixes = 0;

    //     utils::report(
    //         Chapter::Details,
    //         format!(
    //             "Trace: {} does have {} missing spans",
    //             self.base_name,
    //             self.trace.missing_span_ids.len()
    //         ),
    //     );
    //     if let Some(expect_cc) = cchain_cache.get_cchain_key(&self.get_endpoint_key()) {
    //         let new_stats: HashMap<_, _> = mem::take(&mut self.stats_rec.stats)
    //         .into_iter()
    //         .map(|(key, mut stats)| {
    //             let (rooted, mut non_rooted): (Vec<_>, Vec<_>) = stats.call_chain.0
    //                 .into_iter()
    //                 .partition(|(_k2, v2)| v2.rooted);

    //             if !non_rooted.is_empty() {
    //                 let depths: Vec<_> = non_rooted.iter().map(|(_k,v)| v.depth).collect();
    //                 utils::report(Chapter::Details, format!("For key '{key}'  found {} non-rooted out of {} traces with call-chain depths {depths:?}", non_rooted.len(), non_rooted.len() + rooted.len()));
    //             }

    //             // fix the non-rooted paths by a rewrite of the key
    //             let mut fix_failed = 0;
    //             non_rooted.iter_mut()
    //                 .for_each(|(k, v)| {
    //                     if k.remap_callchain(expect_cc) {
    //                         assert!(!v.rooted);  // should be false
    //                         num_fixes += 1;
    //                         v.rooted = true;
    //                     } else {
    //                         fix_failed += 1;
    //                     }});
    //             if fix_failed > 0 {
    //                 utils::report(Chapter::Details, format!("Failed to fix {fix_failed} chains out of {} non-rooted chains. ({num_fixes} fixes applied succesful)", non_rooted.len()));
    //             }

    //             let new_call_chain = rooted.into_iter()
    //                 .chain(non_rooted.into_iter())
    //                 .fold(HashMap::new(), |mut cc, (k, mut v_new)| {
    //                     cc.entry(k)
    //                         .and_modify(|v_curr: &mut CChainStatsValue| {
    //                             v_curr.count += v_new.count;
    //                             v_curr.duration_micros.append(&mut v_new.duration_micros);
    //                         })
    //                         .or_insert(v_new);
    //                     cc
    //                 });
    //             stats.call_chain = CChainStats( new_call_chain );
    //             (key, stats)
    //         })
    //         .collect();
    //         self.stats_rec.stats = new_stats;
    //     } else {
    //         println!("Could not find a call-chain for {}", self.trace.root_call);
    //     };

    //     num_fixes
    // }
}

/// Wrap all traces as a TraceExt to have some additional information available.
pub fn build_trace_ext(traces: Vec<Trace>, folder: &Path, bsr: &BasicStatsRec) -> Vec<TraceExt> {
    // create a traces folder
    let trace_folder = utils::extend_create_folder(folder, "Traces");

    traces
        .into_iter()
        .map(|trace| TraceExt::new(trace, &trace_folder, bsr.clone()))
        .collect::<Vec<_>>()
}
