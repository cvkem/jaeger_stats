use super::{DataPoint, DataSet, LinearRegression};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ExponentialRegression {
    /// the multiplier of the curve y = a * b^x
    pub a: f64,
    /// the base of the exponent in the curve y = a * b^x
    pub b: f64,
    /// Average growth per period (which is derived from b)
    pub avg_growth_per_period: f64,
    /// the R_squared is computed in log-space for the line.
    pub R_squared: f64,
}

impl ExponentialRegression {
    pub fn new(orig_data: &[Option<f64>]) -> Option<Self> {
        let data = get_log_dataset(orig_data);
        let orig_len = orig_data.len();
        match LinearRegression::new_from_dataset(&data, orig_len) {
            Some(lr) => {
                let b = lr.slope.exp();
                Some(Self {
                    a: lr.y_intercept.exp(),
                    b,
                    avg_growth_per_period: b - 1.0,
                    R_squared: lr.R_squared,
                })
            }
            None => None,
        }
    }

    /// predict the y value for a specific x-value
    pub fn predict(&self, x: f64) -> f64 {
        self.a * self.b.powf(x)
    }
}

/// Get the dataset over x and ln(y) for all filled values, where x = counting from 0 to N-1
fn get_log_dataset(data: &[Option<f64>]) -> DataSet {
    data.iter()
        .enumerate()
        // TODO: is  the .as_ref() on next line needed as f64 is copy?
        .filter_map(|(idx, val)| {
            val.as_ref().map(|val| DataPoint {
                x: idx as f64,
                y: (*val).ln(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::ExponentialRegression;

    // helper to compare two floats
    fn match_floats(val: f64, expect: f64) -> bool {
        // do not make bound to strict as we write out fractional values.
        (val - expect).abs() < 1e-10
    }

    fn match_float_opts(val: Option<f64>, expect: Option<f64>) -> bool {
        match val {
            Some(val) => match expect {
                Some(expect) => match_floats(val, expect),
                None => false,
            },
            None => expect.is_none(),
        }
    }

    #[test]
    /// This is a test-case taken from source:
    ///    Input data (0,3),(1,7),(2,10),(3,24),(4,50),(5,95)
    ///    Should return  y=3.0465(1.988)^x
    ///    So a = 3.0465   and b = 1.988
    fn test_case_1() {
        let input = vec![
            Some(3.0),
            Some(7.0),
            Some(10.0),
            Some(24.0),
            Some(50.0),
            Some(95.0),
        ];

        let exp_regr = ExponentialRegression::new(&input).unwrap(); // should exist
        println!("{exp_regr:?}");

        assert!(
            match_floats(exp_regr.a, 3.046450344890837),
            "the multiplier a is incorrect"
        );
        assert!(
            match_floats(exp_regr.b, 1.9880347353739443),
            "The base of the exponent is incorrect"
        );
        assert!(
            match_floats(exp_regr.R_squared, 0.9930119179097666),
            "R_squared incorrect"
        );
    }
}
