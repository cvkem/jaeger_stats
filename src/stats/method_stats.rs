use crate::aux::{format_float, Counted, TimeStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MethodStatsValue {
    pub count: usize,
    pub duration_micros: Vec<i64>,
    pub start_dt_micros: Vec<i64>, // represented via start_dt.timestamp_micros()
    pub num_not_http_ok: i32, // count of the number of call chanis that has one of more HTTP-error(s) somewhere along the chain
    pub num_with_error_logs: i32, // count of the number of call chanis that has one of more ERROR log-lines somewhere along the chain
    pub http_not_ok: Counted<i16>,
    pub error_logs: Counted<String>,
}

impl MethodStatsValue {
    pub fn get_min_millis(&self) -> f64 {
        TimeStats(&self.duration_micros).get_min_millis()
    }

    pub fn get_min_millis_str(&self) -> String {
        TimeStats(&self.duration_micros).get_min_millis_str()
    }

    pub fn get_avg_millis(&self) -> f64 {
        TimeStats(&self.duration_micros).get_avg_millis()
    }

    pub fn get_avg_millis_str(&self) -> String {
        TimeStats(&self.duration_micros).get_avg_millis_str()
    }

    pub fn get_median_millis(&self) -> f64 {
        TimeStats(&self.duration_micros).get_median_millis()
    }

    pub fn get_median_millis_str(&self) -> String {
        TimeStats(&self.duration_micros).get_median_millis_str()
    }

    pub fn get_max_millis(&self) -> f64 {
        TimeStats(&self.duration_micros).get_max_millis()
    }

    pub fn get_max_millis_str(&self) -> String {
        TimeStats(&self.duration_micros).get_max_millis_str()
    }

    pub fn get_avg_rate(&self, num_files: i32) -> Option<f64> {
        TimeStats(&self.start_dt_micros).get_avg_rate(num_files)
    }

    pub fn get_avg_rate_str(&self, num_files: i32) -> String {
        TimeStats(&self.start_dt_micros).get_avg_rate_str(num_files)
    }

    pub fn get_frac_not_http_ok(&self) -> f64 {
        self.num_not_http_ok as f64 / self.count as f64
    }
    pub fn get_frac_not_http_ok_str(&self) -> String {
        format_float(self.get_frac_not_http_ok())
    }

    pub fn get_frac_error_log(&self) -> f64 {
        self.num_with_error_logs as f64 / self.count as f64
    }

    pub fn get_frac_error_log_str(&self) -> String {
        format_float(self.get_frac_error_log())
    }

    /// reports the statistics for a single line
    pub fn report_stats_line(
        &self,
        process_key: &str,
        method: &str,
        n: f64,
        num_files: i32,
    ) -> String {
        let percentage = self.count as f64 / n;
        let expect_duration = percentage * self.get_avg_millis();
        // let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let line = format!(
            "{process_key}/{method}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}",
            self.count,
            self.get_min_millis_str(),
            self.get_median_millis_str(),
            self.get_avg_millis_str(),
            self.get_max_millis_str(),
            format_float(percentage),
            self.get_avg_rate_str(num_files),
            format_float(expect_duration),
            self.get_frac_not_http_ok_str(),
            self.get_frac_error_log_str()
        );
        line
    }
}

/// the information is distributed over the key and the value (no duplication in value)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MethodStats(pub HashMap<String, MethodStatsValue>);
