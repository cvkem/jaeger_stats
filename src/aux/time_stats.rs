//! This module contains some tools om timing statistics, such as averages, min, max, median values.
//! The input is an array of i64 values that represent microseconds. The outputs are metrics in milliseconds.

use super::{calc_rate, format_float, format_float_opt};

pub struct TimeStats<'a>(pub &'a Vec<i64>);

impl<'a> TimeStats<'a> {
    pub fn get_min_millis(&self) -> f64 {
        *self.0.iter().min().expect("Not an integer") as f64 / 1000 as f64
    }

    pub fn get_min_millis_str(&self) -> String {
        format_float(self.get_min_millis())
    }

    pub fn get_median_millis(&self) -> f64 {
        let mut data = self.0.clone();
        data.sort_unstable();
        let len = data.len();
        if len % 2 == 1 {
            data[ len/2 ] as f64 / 1000.0
        } else {
            (data [ len/2 - 1] + data [ len/2 ]) as f64 / 1000.0 / 2.0
        }

    }

    pub fn get_median_millis_str(&self) -> String {
        format_float(self.get_median_millis())
    }

    pub fn get_avg_millis(&self) -> f64 {
        self.0.iter().sum::<i64>() as f64 / (1000 as f64 * self.0.len() as f64)
    }

    pub fn get_avg_millis_str(&self) -> String {
        format_float(self.get_avg_millis())
    }

    pub fn get_max_millis(&self) -> f64 {
        *self.0.iter().max().expect("Not an integer") as f64 / 1000 as f64
    }

    pub fn get_max_millis_str(&self) -> String {
        format_float(self.get_max_millis())
    }

    pub fn get_avg_rate(&self, num_files: i32) -> Option<f64> {
        if let Some((avg_rate, _)) = calc_rate(&self.0, num_files) {
            Some(avg_rate)
        } else {
            None
        }
    }

    pub fn get_avg_rate_str(&self, num_files: i32) -> String {
        format_float_opt(self.get_avg_rate(num_files))
    }

    /// as the distribution is not symmetric (t >/ 0) the median is not a good estimator for the rate as it exludes one tail.
    /// You can expect that the median T (duration between samples) is above the average, and thus the median rate (f = 1/T) is lower.
    pub fn get_median_rate(&self, num_files: i32) -> Option<f64> {
        if let Some((_, median_rate)) = calc_rate(&self.0, num_files) {
            Some(median_rate)
        } else {
            None
        }
    }

    /// as the distribution is not symmetric (t >/ 0) the median is not a good estimator for the rate as it exludes one tail.
    /// You can expect that the median T (duration between samples) is above the average, and thus the median rate (f = 1/T) is lower.
    pub fn get_median_rate_str(&self, num_files: i32) -> String {
        format_float_opt(self.get_median_rate(num_files))
    }
}
