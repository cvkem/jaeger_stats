use crate::aux::{floats_ref_to_string, format_float_opt, LinearRegression};
use std::iter;

#[derive(Debug)]
pub struct StitchedLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub lin_reg: LinearRegression,
}

#[derive(Default, Debug)]
pub struct StitchedSet(pub Vec<StitchedLine>);

impl StitchedLine {
    pub fn avg(&self) -> Option<f64> {
        let values: Vec<_> = self.data.iter().filter_map(|x| *x).collect();
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }

    pub fn scaled_slope(&self) -> Option<f64> {
        self.avg().and_then(|avg| {
            if avg.abs() > 1e-100 {
                self.lin_reg.slope.map(|slope| slope / avg)
            } else {
                None
            }
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
        format!("label; {columns}; ; ; slope; y_intercept; R_squared; scaled_slope")
    }

    /// Show the current line as a string in the csv-format with a ';' separator
    pub fn to_csv_string(&self, header: &str, idx: usize) -> String {
        // Produce the CSV_output
        let values = floats_ref_to_string(&self.data, "; ");

        format!(
            "{header}; {}; {values}; ; ; {}; {}; {}; {}",
            self.label,
            format_float_opt(self.lin_reg.slope),
            format_float_opt(self.lin_reg.y_intercept),
            format_float_opt(self.lin_reg.R_squared),
            format_float_opt(self.scaled_slope()),
        )
    }
}

impl StitchedSet {
    pub fn csv_output(&self, header: &str) -> Vec<String> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, line)| line.to_csv_string(header, idx))
            .collect()
    }

    pub fn summary_header(&self, extra_count: bool) -> Vec<String> {
        let headers = self.0.iter().map(|sl| sl.label.to_owned());
        if extra_count {
            let count_label = self
                .0
                .first()
                .map(|sl| sl.label.to_uppercase())
                .unwrap_or("NO DATA".to_owned());
            iter::once(count_label).chain(headers).collect()
        } else {
            headers.collect()
        }
    }

    pub fn full_data_header(&self) -> String {
        if self.0.is_empty() {
            "NO DATA".to_owned()
        } else {
            self.0[0].headers()
        }
    }

    pub fn summary_avg(&self) -> Vec<Option<f64>> {
        self.0.iter().map(|sl| sl.avg()).collect()
    }

    pub fn summary_slopes(&self) -> Vec<Option<f64>> {
        let count = if self.0.is_empty() {
            None
        } else {
            self.0[0].avg()
        };
        let mut result: Vec<_> = self.0.iter().map(|sl| sl.lin_reg.slope).collect();
        result.insert(0, count);
        result
    }

    pub fn summary_scaled_slopes(&self) -> Vec<Option<f64>> {
        let count = self.0.first().and_then(|sl| sl.avg());
        iter::once(count)
            .chain(self.0.iter().map(|sl| sl.scaled_slope()))
            .collect()
    }
}
