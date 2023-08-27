use crate::utils;

pub struct Anomalies {
    pub slope: Option<f64>,
    pub st_slope: Option<f64>,
    pub l1_deviation: Option<f64>,
}

impl Anomalies {
    /// header for report_stats_line output in ';'-separated csv-format
    pub fn report_stats_line_header_str() -> &'static str {
        "Process; Slope; Short-term slope; L1-deviation"
    }

    pub fn report_stats_line(&self, po: &str) -> String {
        let data = [self.slope, self.st_slope, self.l1_deviation].to_vec();

        format!("{po};{}", utils::floats_to_string(data, "; "))
    }
}
