use crate::utils::{self, Counted, TimeStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProcOperStatsValue {
    pub count: usize,
    /// Duration in microseconds of this Proces/Operation. This includes the full span, so it also covers the waiting-time for (synchronous) downstream calls
    /// num_traces is used, as the name says, to find how many traces use this value.
    /// The other call values below can be inflated in case each trace can call a operation many times.
    pub num_traces: usize,
    pub duration_micros: Vec<i64>,
    /// Represented via start_dt.timestamp_micros(). The end_dt_micros can be derived when adding duration
    pub start_dt_micros: Vec<i64>,
    /// Count of the number of call-chains that has one of more HTTP-error(s) somewhere along the chain
    pub num_not_http_ok: i32,
    /// Count of the number of call-chains that has one of more ERROR log-lines somewhere along the chain (Other log-levels are ignored).
    pub num_with_error_logs: i32,
    /// Contains the actual error-codes that have been observed including the count of these codes
    /// TODO: rename to 'http_not_ok_codes' for clarity. However, this rename will change the file-format.
    pub http_not_ok_codes: Counted<i16>,
    /// Contains the counted list of error-messages that have been observed (Other log-levels are ignored).
    pub error_logs: Counted<String>,
}

impl ProcOperStatsValue {
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

    pub fn get_median_millis(&self) -> Option<f64> {
        TimeStats(&self.duration_micros).get_median_millis()
    }

    /// get the P-percentile over the values
    pub fn get_p_millis(&self, p: f64) -> Option<f64> {
        TimeStats(&self.duration_micros).get_p_millis(p)
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
        utils::format_float(self.get_frac_not_http_ok())
    }

    pub fn get_frac_error_log(&self) -> f64 {
        self.num_with_error_logs as f64 / self.count as f64
    }

    pub fn get_frac_error_log_str(&self) -> String {
        utils::format_float(self.get_frac_error_log())
    }

    /// header for report_stats_line output in ';'-separated csv-format
    pub fn report_stats_line_header_str() -> &'static str {
        "Process/Oper; Count; Num_traces; Min_millis; Avg_millis; Max_millis; Percentage; Rate; Expect_duration; frac_not_http_ok; frac_error_logs"
    }

    /// reports the statistics for a single line in ';'-separated csv-format
    pub fn report_stats_line(
        &self,
        process_key: &str,
        operation: &str,
        n: f64,
        num_files: i32,
    ) -> String {
        let percentage = self.count as f64 / n;
        let expect_duration = percentage * self.get_avg_millis();
        // let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let line = format!(
            "{process_key}/{operation}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}",
            self.count,
            self.num_traces,
            self.get_min_millis_str(),
            self.get_median_millis_str(),
            self.get_avg_millis_str(),
            self.get_max_millis_str(),
            utils::format_float(percentage),
            self.get_avg_rate_str(num_files),
            utils::format_float(expect_duration),
            self.get_frac_not_http_ok_str(),
            self.get_frac_error_log_str()
        );
        line
    }
}

/// the information is distributed over the key and the value (no duplication in value)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProcOperStats(pub HashMap<String, ProcOperStatsValue>);
