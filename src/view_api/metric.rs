use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::mem;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum Metric {
    None = 0,
    NumFiles,
    OccurancePercentage,
    NumTraces,
    NumEndpoints,
    NumIncompleteTraces,
    NumCallChains,
    InitNumUnrootedCallChains,
    NumFixes,
    NumUnrootedCallChainsAfterFixes,
    FracNotHttpOk,
    FracErrorLogs,
    Count,
    Rate,
    MinDurationMillis,
    AvgDurationMillis,
    MedianDurationMillis,
    MaxDurationMillis,
    P75Millis,
    P90Millis,
    P95Millis,
    P99Millis,
}

impl Metric {
    pub fn is_none(&self) -> bool {
        *self == Metric::None
    }

    pub fn to_str(&self) -> &'static str {
        METRIC_LABELS[*self as u8 as usize]
    }
}

/// The Metric_labels should all be set in Lower-case
const METRIC_LABELS: [&str; 22] = [
    "NONE",
    "num_files",
    "occurance percentage",
    "num_traces",
    "num_endpoints",
    "num_incomplete_traces",
    "num_call_chains",
    "init_num_unrooted_cc",
    "num_fixes",
    "num_unrooted_cc_after_fixes",
    "frac_not_http_ok",
    "frac_error_logs",
    "count",
    "rate (req/sec)",
    "minimal duration millis",
    "average duration millis",
    "median duration millis",
    "maximal duration millis",
    "p75 millis",
    "p90 millis",
    "p95 millis",
    "p99 millis",
];

impl ToString for Metric {
    fn to_string(&self) -> String {
        METRIC_LABELS[*self as u8 as usize].to_owned()
    }
}

impl TryFrom<&str> for Metric {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let us = &s.to_lowercase();
        match METRIC_LABELS.iter().position(|&label| label == us) {
            Some(pos) => {
                // TODO: unsafe code needed. It would be better to use a match statement.
                let metric: Metric = unsafe { mem::transmute(pos as u8) };
                Ok(metric)
            }
            None => {
                println!("ERROR: Could not find enum-value for label '{s}'");
                Err("Could not derive Metric for input.")
            }
        }
    }
}
