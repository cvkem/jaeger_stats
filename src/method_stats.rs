
use std::{
    // cmp::Ordering,
    // error::Error,
    // path::Path,
    collections::HashMap};
use crate::{
    stats::{format_float, format_float_opt}, 
    frequency::calculate_frequency};
use serde::{Deserialize, Serialize};


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MethodStatsValue {
    pub count: usize,
    pub duration_micros: Vec<u64>,
    pub start_dt_micros: Vec<i64>,   // represented via start_dt.timestamp_micros()
}



impl MethodStatsValue {
    pub fn new(duration: u64, start_dt_micros: i64) -> Self {
        let duration_micros = [duration].to_vec();
        let start_dt_micros = [start_dt_micros].to_vec();
        Self{count: 1, duration_micros, start_dt_micros}
    }

    /// reports the statistics for a single line
    pub fn report_stats_line(&self, process_key: &str, method: &str, n: f64, num_files: i32) -> String {
        let min_millis = *self.duration_micros.iter().min().expect("Not an integer") as f64 / 1000 as f64;
        let avg_millis = self.duration_micros.iter().sum::<u64>() as f64 / (1000 as f64 * self.duration_micros.len() as f64);
        let max_millis = *self.duration_micros.iter().max().expect("Not an integer") as f64 / 1000 as f64;
        let percentage = self.count as f64 / n;
        let rate = calculate_frequency(&self.start_dt_micros, num_files);
        let expect_duration = percentage * avg_millis;
        // let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let line = format!("{process_key}/{method}; {}; {}; {}; {}; {}; {}; {};", 
            self.count,
            format_float(min_millis), format_float(avg_millis), format_float(max_millis),
            format_float(percentage), format_float_opt(rate), format_float(expect_duration));
        line
    }

}

/// the information is distributed over the key and the value (no duplication in value)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MethodStats  (pub HashMap<String, MethodStatsValue>);

