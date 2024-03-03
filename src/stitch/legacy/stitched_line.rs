use crate::{
    utils::{ExponentialRegression, LinearRegression},
    Metric,
};

use super::{
    super::stitched_line::{BestFit, ShortTermStitchedLine, StitchedLine},
    exponential_regr::LegacyExponentialRegression,
    lineair_regr::LegacyLinearRegression,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
/// StitchedLineLegacy represents a line of values for a certain metric as mentioned in label. When sufficient data is present the Linear regression is added.
/// Also an optional st_line is added which represents the last ST_dATA_LEN values of the data-array including a linear regression. However this is only added if the full data-set contains enough values.
pub struct LegacyStitchedLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub num_filled_columns: u32,
    pub data_avg: Option<f64>,
    pub lin_regr: Option<LinearRegression>,
    pub exp_regr: Option<ExponentialRegression>,
    pub best_fit: BestFit,
    pub st_line: Option<ShortTermStitchedLine>,
}

/// Implement Into directly as we do not want or need/accept the Operation from Stitched to LegacyStitched.
impl TryInto<StitchedLine> for LegacyStitchedLine {
    type Error = &'static str;

    fn try_into(self) -> Result<StitchedLine, Self::Error> {
        let metric = get_metric_from_legacy_str(&self.label[..])?;

        Ok(StitchedLine::new(
            metric,
            self.data,
            self.num_filled_columns,
            self.data_avg,
            self.lin_regr,
            self.exp_regr,
            self.best_fit,
            self.st_line,
        ))
    }
}

#[derive(Debug, Deserialize, Serialize)]
/// StitchedLineLegacyJson represents a line of values for a certain metric as mentioned in label. When sufficient data is present the Linear regression is added.
/// Also an optional st_line is added which represents the last ST_dATA_LEN values of the data-array including a linear regression. However this is only added if the full data-set contains enough values.
/// Here we also can observe lin_regr and exp_regr that only have optional values (legacy)
pub struct LegacyStitchedLineJson {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub num_filled_columns: u32,
    pub data_avg: Option<f64>,
    pub lin_regr: Option<LegacyLinearRegression>,
    pub exp_regr: Option<LegacyExponentialRegression>,
    pub best_fit: BestFit,
    pub st_line: Option<ShortTermStitchedLine>,
}

/// Implement Into directly as we do not want or need/accept the Operation from Stitched to LegacyStitched.
impl TryInto<StitchedLine> for LegacyStitchedLineJson {
    type Error = &'static str;

    fn try_into(self) -> Result<StitchedLine, Self::Error> {
        let metric = get_metric_from_legacy_str(&self.label[..])?;
        let lin_regr = match self.lin_regr {
            Some(lin_regr) => {
                if let Ok(lin_regr) = lin_regr.try_into() {
                    Some(lin_regr)
                } else {
                    None
                }
            }
            None => None,
        };
        let exp_regr = match self.exp_regr {
            Some(exp_regr) => {
                if let Ok(exp_regr) = exp_regr.try_into() {
                    Some(exp_regr)
                } else {
                    None
                }
            }
            None => None,
        };

        Ok(StitchedLine::new(
            metric,
            self.data,
            self.num_filled_columns,
            self.data_avg,
            lin_regr,
            exp_regr,
            self.best_fit,
            self.st_line,
        ))
    }
}

fn get_metric_from_legacy_str(label: &str) -> Result<Metric, &'static str> {
    match label {
        "rate (avg)" => Ok("rate (req/sec)".try_into()?), // special case a 'rate (avg)' and 'rate (req/seq)' are both used in the legacy data.
        "min_millis" | "min_duration_millis" => Ok("minimal duration millis".try_into()?),
        "max_millis" | "max_duration_millis" => Ok("maximal duration millis".try_into()?),
        "avg_duration_millis" => Ok("average duration millis".try_into()?),
        "median_duration_millis" => Ok("median duration millis".try_into()?),
        "p75_millis" => Ok("p75 millis".try_into()?),
        "p90_millis" => Ok("p90 millis".try_into()?),
        "p95_millis" => Ok("p95 millis".try_into()?),
        "p99_millis" => Ok("p99 millis".try_into()?),
        _ => Ok(label.try_into()?),
    }
}
