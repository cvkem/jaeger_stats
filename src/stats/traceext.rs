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
