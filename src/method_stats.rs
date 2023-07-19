
use std::{
    // cmp::Ordering,
    // error::Error,
    // path::Path,
    collections::HashMap};
use crate::stats::format_float;


#[derive(Debug, Default)]
pub struct MethodStatsValue {
    pub count: usize,
    pub duration_micros: Vec<u64>,
}



impl MethodStatsValue {
    pub fn new(duration: u64) -> Self {
        let duration_micros = [duration].to_vec();
        Self{count: 1, duration_micros}
    }

    /// reports the statistics for a single line
    pub fn report_stats_line(&self, process_key: &str, method: &str, n: f64) -> String {
        let min_millis = *self.duration_micros.iter().min().expect("Not an integer") as f64 / 1000 as f64;
        let avg_millis = self.duration_micros.iter().sum::<u64>() as f64 / (1000 as f64 * self.duration_micros.len() as f64);
        let max_millis = *self.duration_micros.iter().max().expect("Not an integer") as f64 / 1000 as f64;
        let freq = self.count as f64 / n;
        let expect_duration = freq * avg_millis;
        // let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let line = format!("{process_key}/{method}; {}; {}; {}; {}; {}; {};", 
            self.count,
            format_float(min_millis), format_float(avg_millis), format_float(max_millis),
            format_float(freq), format_float(expect_duration));
        line
    }

}

/// the information is distributed over the key and the value (no duplication in value)
pub type MethodStats = HashMap<String, MethodStatsValue>;

