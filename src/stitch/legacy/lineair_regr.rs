use crate::utils::LinearRegression;

use serde::{Deserialize, Serialize};

// structure needed to read  old  JSON, but current bincode does not support it.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct LegacyLinearRegression {
    pub slope: Option<f64>,
    pub y_intercept: Option<f64>,
    pub R_squared: Option<f64>,
    pub L1_deviation: Option<f64>,
    pub avg_growth_per_period: Option<f64>,
}

/// Implement Into directly as we do not want or need/accept the Operation from Stitched to LegacyStitched.
impl TryInto<LinearRegression> for LegacyLinearRegression {
    type Error = &'static str;

    fn try_into(self) -> Result<LinearRegression, Self::Error> {
        if self.slope == None
            && self.y_intercept == None
            && self.R_squared == None
            && self.L1_deviation == None
        {
            Err("No Linear-regression")
        } else {
            Ok(LinearRegression {
                slope: self.slope.unwrap(),
                y_intercept: self.y_intercept.unwrap(),
                R_squared: self.R_squared.unwrap(),
                L1_deviation: self.L1_deviation.unwrap(),
                avg_growth_per_period: self.avg_growth_per_period,
            })
        }
    }
}
