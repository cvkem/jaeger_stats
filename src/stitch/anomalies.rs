use crate::utils;

use super::stitched_line::StitchedLine;

#[derive(Debug)]
pub struct AnomalyParameters {
    pub scaled_slope_bound: f64,    // default 0.05
    pub st_num_points: usize,       // default 5
    pub scaled_st_slope_bound: f64, // default 0.05
    pub l1_dev_bound: f64,          // default 2.0
}
pub struct Anomalies {
    pub scaled_slope: Option<f64>,
    pub st_scaled_slope: Option<f64>,
    pub l1_deviation: Option<f64>,
}

impl Anomalies {
    pub fn anomalies(line: &StitchedLine, pars: &AnomalyParameters) -> Option<Anomalies> {
        line.lin_reg.as_ref().and_then(|_lin_reg| {
            let scaled_slope = line.scaled_slope().and_then(|sslope| {
                if sslope > pars.scaled_slope_bound {
                    Some(sslope)
                } else {
                    None
                }
            });
            let st_scaled_slope = line.scaled_st_slope().and_then(|sslope| {
                if sslope > pars.scaled_st_slope_bound {
                    Some(sslope)
                } else {
                    None
                }
            });
            let l1_deviation = line.last_deviation_scaled().and_then(|l1_dev| {
                if l1_dev > pars.l1_dev_bound {
                    Some(l1_dev)
                } else {
                    None
                }
            });

            if scaled_slope.is_some() || st_scaled_slope.is_some() || l1_deviation.is_some() {
                Some(Anomalies {
                    scaled_slope,
                    st_scaled_slope,
                    l1_deviation,
                })
            } else {
                None
            }
        })
    }

    /// header for report_stats_line output in ';'-separated csv-format
    pub fn report_stats_line_header_str() -> &'static str {
        "Process; Scaled_slope; Short-term scaled_slope; L1-deviation"
    }

    pub fn report_stats_line(&self, po: &str) -> String {
        let data = [self.scaled_slope, self.st_scaled_slope, self.l1_deviation].to_vec();

        format!("{po};{}", utils::floats_to_string(data, "; "))
    }
}
