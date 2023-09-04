use super::anomalies::{Anomalies, AnomalyParameters};
use crate::utils::{self, LinearRegression};

const MIN_POINTS_FOR_ST_MULTIPLIER: usize = 2;

#[derive(Debug)]
/// Used to represent a short time-interval and the linear regression on this short interval. This data is used to detect anomalies
pub struct ShortTermStitchedLine {
    pub data: Vec<Option<f64>>,
    pub lin_reg: LinearRegression,
}

#[derive(Debug)]
/// StitchedLine represents a line of values for a certain metric as mentioned in label. When sufficient data is present the Linear regression is added.
/// Also an optional st_line is added which represents the last ST_dATA_LEN values of the data-array including a linear regression. However this is only added if the full data-set contains enough values.
pub struct StitchedLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub num_filled_columns: u32,
    pub data_avg: Option<f64>,
    pub lin_reg: Option<LinearRegression>,
    pub st_line: Option<ShortTermStitchedLine>,
}

impl StitchedLine {
    /// compute a data-series (StitchedLine) including linear regression, the average of the data and possibly a short-term line for the last few datapoint.
    /// The Short-Term line is used to detect anomalies. However, this is only computed if the full dataset significantly exceed the size of the ST
    pub fn new(label: String, data: Vec<Option<f64>>, pars: &AnomalyParameters) -> Self {
        let lin_reg = LinearRegression::new(&data);

        let st_line = if data.len() >= MIN_POINTS_FOR_ST_MULTIPLIER * pars.st_num_points {
            let st_data: Vec<_> = data
                .iter()
                .skip(data.len() - pars.st_num_points)
                .copied()
                .collect();
            // Only if the Lineair regression is possible a ShortTermStitchedLine is returne
            LinearRegression::new(&st_data).map(|lr| ShortTermStitchedLine {
                data: st_data,
                lin_reg: lr,
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
            lin_reg,
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

    /// Compute a scaled slope by moving the average value to 0.5.
    /// This will scale down the slope as if data stems from the interval [0,1], provided data has a symetric distribution.
    pub fn scaled_slope(&self) -> Option<f64> {
        self.data_avg.and_then(|avg| {
            if avg.abs() > 1e-100 {
                self.lin_reg
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
                    .map(|stl| stl.lin_reg.slope / (2.0 * avg))
            } else {
                None
            }
        })
    }

    pub fn last_deviation_scaled(&self) -> Option<f64> {
        self.lin_reg.as_ref().and_then(|lr| {
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

    /// return the headers that correspond to a data-row (assuming this Line is representative for data availability over the full dataset).
    pub fn headers(&self) -> String {
        let columns = self
            .data
            .iter()
            .enumerate()
            .map(|(idx, x)| {
                if x.is_some() {
                    format!("{}", idx + 1)
                } else {
                    format!("_{}", idx + 1)
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("label; num_col; {columns}; ; ; slope; y_intercept; R_squared; L1_deviation, scaled_slope, last_deviation")
    }

    /// Show the current line as a string in the csv-format with a ';' separator
    pub fn to_csv_string(&self, prefixes: &[&str]) -> String {
        // Produce the CSV_output
        let values = utils::floats_ref_to_string(&self.data, "; ");
        let header = prefixes.join("; ");

        if let Some(lr) = &self.lin_reg {
            format!(
                "{header}; {}; {}; {values}; ; ; {}; {}; {}; {}; {}; {}",
                self.label,
                self.num_filled_columns,
                utils::format_float(lr.slope),
                utils::format_float(lr.y_intercept),
                utils::format_float(lr.R_squared),
                utils::format_float(lr.L1_deviation),
                utils::format_float_opt(self.scaled_slope()),
                utils::format_float_opt(self.last_deviation_scaled()),
            )
        } else {
            format!("{header}; {}; {values}; ; ; ; ; ;", self.label,)
        }
    }
}
