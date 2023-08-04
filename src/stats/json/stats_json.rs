// TODO move to Stats
use crate::stats::{
        call_chain::{CChainStatsKey, CChainStatsValue},
        MethodStats,
        Stats, StatsRec};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    fs::File,
    io
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatsJson {
    pub num_received_calls: usize, // inbound calls to this process
    pub num_outbound_calls: usize, // outbound calls to other processes
    pub num_unknown_calls: usize,
    pub method: MethodStats,
    //    method_cache_suffix: HashMap<String, usize>,  // methods in a cache-chain have a suffix.
    pub call_chain: Vec<(CChainStatsKey, CChainStatsValue)>,
}

impl From<Stats> for StatsJson {
    fn from(st: Stats) -> Self {
        StatsJson {
            num_received_calls: st.num_received_calls,
            num_outbound_calls: st.num_outbound_calls,
            num_unknown_calls: st.num_unknown_calls,
            method: st.method,
            call_chain: st.call_chain.into_iter().collect(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatsRecJson {
    pub trace_id: Vec<String>,
    pub root_call: Vec<String>,
    pub num_spans: Vec<usize>,
    pub num_files: Option<i32>, //Optional for backward compatibility
    pub start_dt: Vec<String>,
    pub end_dt: Vec<String>,
    pub duration_micros: Vec<u64>,
    pub time_to_respond_micros: Vec<u64>,
    pub caching_process: Vec<String>,
    pub stats: HashMap<String, StatsJson>, // hashmap base on the leaf process (as that is the initial level of reporting)
}

impl From<StatsRec> for StatsRecJson {
    fn from(sr: StatsRec) -> Self {
        let stats: HashMap<String, StatsJson> =
            sr.stats.into_iter().map(|(k, v)| (k, v.into())).collect();
        StatsRecJson {
            trace_id: sr.trace_id,
            root_call: sr.root_call,
            num_spans: sr.num_spans,
            num_files: Some(sr.num_files), // Made optional for backward compatibility
            start_dt: sr.start_dt.into_iter().map(|dt| dt.to_string()).collect(),
            end_dt: sr.end_dt.into_iter().map(|dt| dt.to_string()).collect(),
            duration_micros: sr.duration_micros,
            time_to_respond_micros: sr.time_to_respond_micros,
            caching_process: sr.caching_process,
            stats: stats,
        }
    }
}

impl StatsRecJson {
    /// StatsJson file and parse it
    pub fn read_file(path: &OsString) -> Result<Self, Box<dyn Error>> {
        let f = File::open(path)?;
        let reader = io::BufReader::new(f);
        let sj = serde_json::from_reader(reader)?;
        Ok(sj)
    }
}
