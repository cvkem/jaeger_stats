
use std::{
    // cmp::Ordering,
    // error::Error,
    // path::Path,
    collections::HashMap};
use crate::{
    stats::{format_float, format_float_opt}, 
    rate::calc_rate};
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

    pub fn get_min_millis_str(&self) -> String {
        let min_millis = *self.duration_micros.iter().min().expect("Not an integer") as f64 / 1000 as f64;
        format_float(min_millis)
    }

    pub fn get_avg_millis(&self) -> f64 {
        self.duration_micros.iter().sum::<u64>() as f64 / (1000 as f64 * self.duration_micros.len() as f64)
    }
    
    pub fn get_avg_millis_str(&self) -> String {
        format_float(self.get_avg_millis())
    }

    pub fn get_max_millis_str(&self) -> String {
        let max_millis = *self.duration_micros.iter().max().expect("Not an integer") as f64 / 1000 as f64;
        format_float(max_millis)
    }

    pub fn get_rate_str(&self, num_files: i32) -> String {
        let rate = calc_rate(&self.start_dt_micros, num_files);
        format_float_opt(rate)
    }

    /// reports the statistics for a single line
    pub fn report_stats_line(&self, process_key: &str, method: &str, n: f64, num_files: i32) -> String {
        let percentage = self.count as f64 / n;
        let expect_duration = percentage * self.get_avg_millis();
        // let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let line = format!("{process_key}/{method}; {}; {}; {}; {}; {}; {}; {};", 
            self.count, self.get_min_millis_str(), self.get_avg_millis_str(), self.get_max_millis_str(),
            format_float(percentage), self.get_rate_str(num_files), format_float(expect_duration));
        line
    }

}

/// the information is distributed over the key and the value (no duplication in value)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MethodStats  (pub HashMap<String, MethodStatsValue>);

