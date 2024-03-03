use crate::utils::{ExponentialRegression, LinearRegression};

use super::super::stitched_line::{BestFit, ShortTermStitchedLine, StitchedLine};
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
        let metric = match &self.label[..] {
            "rate (avg)" => "rate (req/sec)".try_into()?, // special case a 'rate (avg)' and 'rate (req/seq)' are both used in the legacy data.
            "min_millis" | "min_duration_millis" => "minimal duration millis".try_into()?,
            "max_millis" | "max_duration_millis" => "maximal duration millis".try_into()?,
            "avg_duration_millis" => "average duration millis".try_into()?,
            "median_duration_millis" => "median duration millis".try_into()?,
            "p75_millis" => "p75 millis".try_into()?,
            "p90_millis" => "p90 millis".try_into()?,
            "p95_millis" => "p95 millis".try_into()?,
            "p99_millis" => "p99 millis".try_into()?,
            _ => (&self.label[..]).try_into()?,
        };

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
