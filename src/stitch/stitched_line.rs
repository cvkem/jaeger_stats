use super::anomalies::Anomalies;
use crate::utils::{self, LinearRegression};
use std::iter;

const ST_DATA_LEN: usize = 5;
const MIN_SL_LEN_FOR_ST_LINE: usize = 10;

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
    pub data_avg: Option<f64>,
    pub lin_reg: Option<LinearRegression>,
    pub st_line: Option<ShortTermStitchedLine>,
}

impl StitchedLine {
    pub fn new(label: String, data: Vec<Option<f64>>) -> Self {
        let lin_reg = LinearRegression::new(&data);

        let st_line = if data.len() >= MIN_SL_LEN_FOR_ST_LINE {
            let st_data: Vec<_> = data
                .iter()
                .skip(data.len() - ST_DATA_LEN)
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

        Self {
            label,
            data,
            data_avg,
            lin_reg,
            st_line,
        }
    }

    pub fn anomalies(&self) -> Option<Anomalies> {
        self.lin_reg.as_ref().and_then(|_lin_reg| {
            let slope =
                self.scaled_slope()
                    .and_then(|sslope| if sslope > 0.05 { Some(sslope) } else { None });
            let st_slope =
                self.scaled_st_slope()
                    .and_then(|sslope| if sslope > 0.05 { Some(sslope) } else { None });
            let l1_deviation = self.last_deviation_scaled().and_then(|l1_dev| {
                if l1_dev > 2.0 {
                    Some(l1_dev)
                } else {
                    None
                }
            });

            if slope.is_some() || st_slope.is_some() || l1_deviation.is_some() {
                Some(Anomalies {
                    slope,
                    st_slope,
                    l1_deviation,
                })
            } else {
                None
            }
        })
    }

    pub fn calculate_avg(data: &Vec<Option<f64>>) -> Option<f64> {
        let values: Vec<_> = data.iter().filter_map(|x| *x).collect();
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }

    pub fn scaled_slope(&self) -> Option<f64> {
        self.data_avg.and_then(|avg| {
            if avg.abs() > 1e-100 {
                self.lin_reg.as_ref().map(|lin_reg| lin_reg.slope / avg)
            } else {
                None
            }
        })
    }

    pub fn scaled_st_slope(&self) -> Option<f64> {
        self.data_avg.and_then(|avg| {
            if avg.abs() > 1e-100 {
                self.st_line.as_ref().map(|stl| stl.lin_reg.slope / avg)
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
        format!("label; {columns}; ; ; slope; y_intercept; R_squared; L1_deviatipn, scaled_slope, last_deviation")
    }

    /// Show the current line as a string in the csv-format with a ';' separator
    pub fn to_csv_string(&self, header: &str, idx: usize) -> String {
        // Produce the CSV_output
        let values = utils::floats_ref_to_string(&self.data, "; ");

        if let Some(lr) = &self.lin_reg {
            format!(
                "{header}; {}; {values}; ; ; {}; {}; {}; {}; {}; {}",
                self.label,
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
