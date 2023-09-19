//! Variant on StatsRec that can be stored in JSON format.
//! The StatsRec can not be stored in JSON as:
//!    1. It contains a HashMap (call-chain) with non-string keys
//!    2. It contains date-times which can not represented in JSON (we will store them as a i64, just like we had in the Jaeger-JSON file)
use crate::{
    stats::{
        call_chain::{CChainStatsKey, CChainStatsValue},
        OperationStats, ProcOperStats, StatsRec, Version,
    },
    utils,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, ffi::OsString, fs::File, io, path::Path};

/// The OperationStatsJson is used as an intermediate value for storage as JSON does not allow compound hashmap-keys.
/// Thus Hashmap is flattened to a vector of key-value pairs. For more details on the fields see OperationStats.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OperationStatsJson {
    pub method: ProcOperStats,
    pub num_traces: usize,
    pub num_received_calls: usize, // inbound calls to this process
    pub num_outbound_calls: usize, // outbound calls to other processes
    pub num_unknown_calls: usize,
    //    method_cache_suffix: HashMap<String, usize>,  // methods in a cache-chain have a suffix.
    pub call_chain: Vec<(CChainStatsKey, CChainStatsValue)>,
}

impl From<OperationStats> for OperationStatsJson {
    fn from(st: OperationStats) -> Self {
        Self {
            method: st.operation,
            num_traces: st.num_traces,
            num_received_calls: st.num_received_calls,
            num_outbound_calls: st.num_outbound_calls,
            num_unknown_calls: st.num_unknown_calls,
            call_chain: st.call_chain.0.into_iter().collect(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatsRecJson {
    pub version: Version,
    pub trace_id: Vec<String>,
    pub root_call: Vec<String>,
    pub num_spans: Vec<usize>,
    pub num_files: i32,
    /// number of endpoint included
    pub num_endpoints: usize,
    /// number of incomplete traces after application of the fixes
    pub num_incomplete_traces: usize,
    pub num_call_chains: usize,

    // initial number of Call-chains that do not start at the root of the full trace
    pub init_num_unrooted_cc: usize,
    /// number of fixes applied
    pub num_fixes: usize,
    // Number of Call-chains that do not start at the root of the full trace after fixes based on call-chains
    pub num_unrooted_cc_after_fixes: usize,
    pub start_dt: Vec<i64>,
    pub end_dt: Vec<i64>,
    pub duration_micros: Vec<i64>,
    pub time_to_respond_micros: Vec<i64>,
    pub caching_processes: Vec<String>,
    pub stats: HashMap<String, OperationStatsJson>, // hashmap base on the leaf process (as that is the initial level of reporting)
}

impl From<StatsRec> for StatsRecJson {
    fn from(sr: StatsRec) -> Self {
        let stats: HashMap<String, OperationStatsJson> =
            sr.stats.into_iter().map(|(k, v)| (k, v.into())).collect();
        Self {
            version: sr.version,
            trace_id: sr.trace_id,
            root_call: sr.root_call,
            num_spans: sr.num_spans,
            num_files: sr.num_files,
            num_endpoints: sr.num_endpoints,
            num_incomplete_traces: sr.num_incomplete_traces,
            num_call_chains: sr.num_call_chains,
            init_num_unrooted_cc: sr.init_num_unrooted_cc,
            num_fixes: sr.num_fixes,
            num_unrooted_cc_after_fixes: sr.num_unrooted_cc_after_fixes,
            start_dt: sr
                .start_dt
                .into_iter()
                .map(utils::datetime_to_micros)
                .collect(),
            end_dt: sr
                .end_dt
                .into_iter()
                .map(utils::datetime_to_micros)
                .collect(),
            duration_micros: sr.duration_micros,
            time_to_respond_micros: sr.time_to_respond_micros,
            caching_processes: sr.caching_processes,
            stats,
        }
    }
}

impl StatsRecJson {
    /// StatsJson file and parse it
    pub fn read_file(path: &OsString) -> Result<Self, Box<dyn Error>> {
        let keep = path.clone().into_string().unwrap();
        let path_str = Path::new(&keep);
        let f = File::open(path)?;
        let reader = io::BufReader::new(f);

        let Some(ext) = path_str.extension() else {
            panic!("Failed to find extension of '{}'", path_str.display());
        };
        let ext = ext.to_str().unwrap();

        let sj = match ext {
            "json" => serde_json::from_reader(reader)?,
            "bincode" => bincode::deserialize_from(reader)?,
            ext => panic!(
                "Unknown extension '{ext}'of inputfile {}",
                path_str.display()
            ),
        };
        Ok(sj)
    }
}
