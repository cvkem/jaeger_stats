use crate::utils::ExponentialRegression;

use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct LegacyExponentialRegression {
    /// the multiplier of the curve y = a * b^x
    pub a: Option<f64>,
    /// the base of the exponent in the curve y = a * b^x
    pub b: Option<f64>,
    /// Average growth per period (which is derived from b)
    pub avg_growth_per_period: Option<f64>,
    /// the R_squared is computed in log-space for the line.
    pub R_squared: Option<f64>,
}

/// Implement Into directly as we do not want or need/accept the Operation from Stitched to LegacyStitched.
impl TryInto<ExponentialRegression> for LegacyExponentialRegression {
    type Error = &'static str;

    fn try_into(self) -> Result<ExponentialRegression, Self::Error> {
        if self.a == None
            && self.b == None
            && self.R_squared == None
            && self.avg_growth_per_period == None
        {
            Err("No Exponential-regression")
        } else {
            Ok(ExponentialRegression {
                a: self.a.unwrap(),
                b: self.b.unwrap(),
                R_squared: self.R_squared.unwrap(),
                avg_growth_per_period: self.avg_growth_per_period.unwrap(),
            })
        }
    }
}
