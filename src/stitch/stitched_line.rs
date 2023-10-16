use super::anomalies::{Anomalies, AnomalyParameters};
use crate::utils::{self, ExponentialRegression, LinearRegression};
use serde::{Deserialize, Serialize};

const MIN_POINTS_FOR_ST_MULTIPLIER: usize = 2;

#[derive(Debug, Deserialize, Serialize)]
/// Used to represent a short time-interval and the linear regression on this short interval. This data is used to detect anomalies
pub struct ShortTermStitchedLine {
    pub data: Vec<Option<f64>>,
    pub lin_regr: LinearRegression,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BestFit {
    LinRegr,
    ExprRegr,
    None,
}

impl ToString for BestFit {
    fn to_string(&self) -> String {
        match self {
            BestFit::LinRegr => "Linear".to_string(),
            BestFit::ExprRegr => "Exponential".to_string(),
            BestFit::None => String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
/// StitchedLine represents a line of values for a certain metric as mentioned in label. When sufficient data is present the Linear regression is added.
/// Also an optional st_line is added which represents the last ST_dATA_LEN values of the data-array including a linear regression. However this is only added if the full data-set contains enough values.
pub struct StitchedLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub num_filled_columns: u32,
    pub data_avg: Option<f64>,
    pub lin_regr: Option<LinearRegression>,
    pub exp_regr: Option<ExponentialRegression>,
    pub best_fit: BestFit,
    pub st_line: Option<ShortTermStitchedLine>,
}

impl StitchedLine {
    /// compute a data-series (StitchedLine) including linear regression, the average of the data and possibly a short-term line for the last few datapoint.
    /// The Short-Term line is used to detect anomalies. However, this is only computed if the full dataset significantly exceed the size of the ST
    pub fn new(label: String, data: Vec<Option<f64>>, pars: &AnomalyParameters) -> Self {
        let lin_regr = LinearRegression::new(&data);
        let exp_regr = ExponentialRegression::new(&data);
        let best_fit = match (&lin_regr, &exp_regr) {
            (None, None) => BestFit::None,
            (Some(_), None) => BestFit::LinRegr,
            (None, Some(_)) => BestFit::ExprRegr,
            (Some(lr), Some(er)) => {
                if er.R_squared > lr.R_squared {
                    BestFit::ExprRegr
                } else {
                    BestFit::LinRegr
                }
            }
        };

        let st_line = if data.len() >= MIN_POINTS_FOR_ST_MULTIPLIER * pars.st_num_points {
            let st_data: Vec<_> = data
                .iter()
                .skip(data.len() - pars.st_num_points)
                .copied()
                .collect();
            // Only if the Lineair regression is possible a ShortTermStitchedLine is returne
            LinearRegression::new(&st_data).map(|lr| ShortTermStitchedLine {
                data: st_data,
                lin_regr: lr,
            })
        } else {
            None
        };

        let data_avg = Self::calculate_avg(&data);
        let num_filled_columns = data
            .iter()
            .fold(0, |cnt, val| if val.is_some() { cnt + 1 } else { cnt });

        Self {
            label,
            data,
            num_filled_columns,
            data_avg,
            lin_regr,
            exp_regr,
            best_fit,
            st_line,
        }
    }

    pub fn anomalies(&self, pars: &AnomalyParameters) -> Option<Anomalies> {
        Anomalies::anomalies(&self, pars)
    }

    pub fn calculate_avg(data: &Vec<Option<f64>>) -> Option<f64> {
        let values: Vec<_> = data.iter().filter_map(|x| *x).collect();
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }

    /// Compute the growth per time-interval
    pub fn periodic_growth(&self) -> Option<f64> {
        match &self.best_fit {
            BestFit::LinRegr => self
                .lin_regr
                .as_ref()
                .and_then(|lr| lr.avg_growth_per_period),
            BestFit::ExprRegr => self
                .exp_regr
                .as_ref()
                .and_then(|er| Some(er.avg_growth_per_period)),
            _ => None,
        }
    }

    /// Compute a scaled slope by moving the average value to 0.5.
    /// This will scale down the slope as if data stems from the interval [0,1], provided data has a symetric distribution.
    pub fn scaled_slope(&self) -> Option<f64> {
        self.data_avg.and_then(|avg| {
            if avg.abs() > 1e-100 {
                self.lin_regr
                    .as_ref()
                    .map(|lin_reg| lin_reg.slope / (2.0 * avg))
            } else {
                None
            }
        })
    }

    /// Compute a scaled slope on the short term data by moving the average value to 0.5.
    /// This will scale down the slope as if data stems from the interval [0,1], provided data has a symetric distribution.
    /// The average of the full dataset is used for the scaling, and not the average of the short-term daata.
    pub fn scaled_st_slope(&self) -> Option<f64> {
        self.data_avg.and_then(|avg| {
            if avg.abs() > 1e-100 {
                self.st_line
                    .as_ref()
                    .map(|stl| stl.lin_regr.slope / (2.0 * avg))
            } else {
                None
            }
        })
    }

    pub fn last_deviation_scaled(&self) -> Option<f64> {
        self.lin_regr.as_ref().and_then(|lr| {
            lr.get_deviation(&self.data, self.data.len() - 1)
                .and_then(|deviation| {
                    if lr.L1_deviation.abs() > 1e-100 {
                        Some(deviation / lr.L1_deviation)
                    } else {
                        None
                    }
                })
        })
    }

    fn last_exp_model_deviation(&self) -> Option<f64> {
        self.exp_regr.as_ref().and_then(|er| {
            self.data[self.data.len() - 1].map(|val| val - er.predict((self.data.len() - 1) as f64))
        })
    }

    /// return the headers that correspond to a data-row (assuming this Line is representative for data availability over the full dataset).
    pub fn headers(&self) -> String {
        let columns = self
            .data
            .iter()
            .enumerate()
            .map(|(idx, x)| {
                if x.is_some() {
                    format!("{}", idx)
                } else {
                    format!("_{}", idx)
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("label; NUM_FILLED; {columns}; ; ; best_fit; slope; y_intercept; r2; L1_norm; scaled_slope; last_deviation; periodic_growth; exp_a, exp_b; exp_r2; exp_last_dev;")
    }

    /// Show the current line as a string in the csv-format with a ';' separator
    pub fn to_csv_string(&self, prefixes: &[&str]) -> String {
        // Produce the CSV_output
        let values = utils::floats_ref_to_string(&self.data, "; ");
        let header = prefixes.join("; ");

        let (exp_a, exp_b, exp_r2) = match &self.exp_regr {
            Some(er) => (Some(er.a), Some(er.b), Some(er.R_squared)),
            None => (None, None, None),
        };

        if let Some(lr) = &self.lin_regr {
            format!(
                "{header}; {}; {}; {values}; ; ; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {};",
                self.label,
                self.num_filled_columns,
                self.best_fit.to_string(),
                utils::format_float(lr.slope),
                utils::format_float(lr.y_intercept),
                utils::format_float(lr.R_squared),
                utils::format_float(lr.L1_deviation),
                utils::format_float_opt(self.scaled_slope()),
                utils::format_float_opt(self.last_deviation_scaled()),
                utils::format_float_opt(self.periodic_growth()),
                utils::format_float_opt(exp_a),
                utils::format_float_opt(exp_b),
                utils::format_float_opt(exp_r2),
                utils::format_float_opt(self.last_exp_model_deviation())
            )
        } else {
            format!(
                "{header}; {}; {}; {values}; ; ; ; ; ;",
                self.label, self.num_filled_columns
            )
        }
    }

    // /// Get a subset of selected data-points for each of the stiched lines in the stitched set.
    // /// assume that the size of the selection was checked by the upstream process (the caller).
    // pub fn get_selection(&self, selection: &Vec<bool>) -> Self {
    // }
}
