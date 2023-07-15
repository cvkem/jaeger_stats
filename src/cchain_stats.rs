use std::collections::HashMap;
use crate::{call_chain::{CallChain, call_chain_key},
    stats::format_float};


#[derive(Debug, Default)]
pub struct CChainStatsValue {
//    pub method: String,
    pub count: usize,
    pub depth: usize,
    pub duration_micros: Vec<u64>,
    pub looped: Vec<String>,
}


/// Key for the CChain containing part of the CChain-values 
#[derive(Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct CChainStatsKey {
    pub call_chain: CallChain,
    pub caching_process: String,  // an empty string or a one or more caching-processes between square brackets
    pub is_leaf: bool,
}

impl CChainStatsKey {

    /// get the method of the current call (last call in the call-chain)
    pub fn get_method(&self) -> &str {
        &self.call_chain[self.call_chain.len()-1].method
    }

    pub fn call_chain_key(&self) -> String {
        call_chain_key(&self.call_chain, &self.caching_process, self.is_leaf)
    }
}


impl CChainStatsValue {
    pub fn new() -> Self {
        Default::default()
    }

    /// reports the statistics for a single line
    pub fn report_stats_line(&self, process_key: &str, ps_key: &CChainStatsKey, n: f64) -> String {
        let min_millis = *self.duration_micros.iter().min().expect("Not an integer") as f64 / 1000 as f64;
        let avg_millis = self.duration_micros.iter().sum::<u64>() as f64 / (1000 as f64 * self.duration_micros.len() as f64);
        let max_millis = *self.duration_micros.iter().max().expect("Not an integer") as f64 / 1000 as f64;
        let method = ps_key.get_method();
        let caching_process = &ps_key.caching_process;
        let freq = self.count as f64 / n;
        let expect_duration = freq * avg_millis;
        let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let line = format!("{process_key}/{method} {caching_process}; {}; {}; {}; {}; {:?}; {}; {}; {}; {}; {}; {}; {}", 
            ps_key.is_leaf, self.depth, self.count, self.looped.len()> 0, 
            self.looped, ps_key.call_chain_key(), 
            format_float(min_millis), format_float(avg_millis), format_float(max_millis),
            format_float(freq), format_float(expect_duration), format_float(expect_contribution));
        line
    }

}

/// the information is distributed over the key and the value (no duplication in value)
pub type CChainStats = HashMap<CChainStatsKey, CChainStatsValue>;

