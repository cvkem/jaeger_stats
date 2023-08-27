//! This module contains some tools om timing statistics, such as averages, min, max, median values.
//! The input is an array of i64 values that represent microseconds. The outputs are metrics in milliseconds.

#[allow(dead_code)]

/// Wrapper to implement time-functions on an array of integers that represent times in nicro-seconds.
/// The derive time-values are floats in milliseconds
pub struct TimeStats<'a>(pub &'a Vec<i64>);

impl<'a> TimeStats<'a> {
    pub fn get_min_millis(&self) -> f64 {
        *self.0.iter().min().expect("Not an integer") as f64 / 1000_f64
    }

    pub fn get_min_millis_str(&self) -> String {
        super::format_float(self.get_min_millis())
    }

    /// Computation of a P-percentile value, which is an exiting value that exceed P% of the measured values.
    pub fn get_p_millis(&self, p: f64) -> Option<f64> {
        let mut data = self.0.clone();
        data.sort_unstable();
        let idx = ((data.len()) as f64 * p).ceil() as usize - 1; // .ceil() used to err on the safe side. Our index starts at 0 instead of 1, so correct with -1
        if idx >= data.len() - 1 {
            None
        } else {
            Some(data[idx] as f64 / 1000.0)
        }
    }

    pub fn get_p_millis_str(&self, p: f64) -> String {
        super::format_float_opt(self.get_p_millis(p))
    }

    /// Determine medium in milliseconds. For an odd sized array the Median equals the p50 value (percentile 50).
    pub fn get_median_millis(&self) -> Option<f64> {
        if self.0.len() < 3 {
            None
        } else {
            let result = {
                let mut data = self.0.clone();
                data.sort_unstable();
                let len = data.len();
                if len % 2 == 1 {
                    data[len / 2] as f64 / 1000.0
                } else {
                    (data[len / 2 - 1] + data[len / 2]) as f64 / 1000.0 / 2.0
                }
            };
            Some(result)
        }
    }

    pub fn get_median_millis_str(&self) -> String {
        super::format_float_opt(self.get_median_millis())
    }

    pub fn get_avg_millis(&self) -> f64 {
        self.0.iter().sum::<i64>() as f64 / (1000_f64 * self.0.len() as f64)
    }

    pub fn get_avg_millis_str(&self) -> String {
        super::format_float(self.get_avg_millis())
    }

    pub fn get_max_millis(&self) -> f64 {
        *self.0.iter().max().expect("Not an integer") as f64 / 1000_f64
    }

    pub fn get_max_millis_str(&self) -> String {
        super::format_float(self.get_max_millis())
    }

    pub fn get_avg_rate(&self, num_files: i32) -> Option<f64> {
        if let Some((avg_rate, _)) = super::calc_rate(self.0, num_files) {
            Some(avg_rate)
        } else {
            None
        }
    }

    pub fn get_avg_rate_str(&self, num_files: i32) -> String {
        super::format_float_opt(self.get_avg_rate(num_files))
    }

    /// as the distribution is not symmetric (t >/ 0) the median is not a good estimator for the rate as it exludes one tail.
    /// You can expect that the median T (duration between samples) is above the average, and thus the median rate (f = 1/T) is lower.
    pub fn get_median_rate(&self, num_files: i32) -> Option<f64> {
        if let Some((_, median_rate)) = super::calc_rate(self.0, num_files) {
            Some(median_rate)
        } else {
            None
        }
    }

    /// as the distribution is not symmetric (t >/ 0) the median is not a good estimator for the rate as it exludes one tail.
    /// You can expect that the median T (duration between samples) is above the average, and thus the median rate (f = 1/T) is lower.
    pub fn get_median_rate_str(&self, num_files: i32) -> String {
        super::format_float_opt(self.get_median_rate(num_files))
    }
}

#[cfg(test)]
mod tests {
    use super::TimeStats;

    fn match_floats(val: f64, expect: f64) -> bool {
        (val - expect).abs() < 1e-100
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
    fn median_ordered_odd_count() {
        let input = vec![1000_i64, 2000, 6000];
        let avg = TimeStats(&input).get_avg_millis();
        let median = TimeStats(&input).get_median_millis();
        let p90 = TimeStats(&input).get_p_millis(0.9);

        println!("avg={avg}  and median={median}   and p90={p90:?}");

        println!("avg={avg}  and median={median}");

        assert!(match_floats(avg, 3.0));
        assert!(match_floats(median, 2.0));
        assert!(p90.is_none());
    }

    #[test]
    fn median_ordered_even_count() {
        let input = vec![1000_i64, 2000, 3000, 6000];
        let avg = TimeStats(&input).get_avg_millis();
        let median = TimeStats(&input).get_median_millis();

        println!("avg={avg}  and median={median}");

        assert!(match_floats(avg, 3.0));
        assert!(match_floats(median, 2.5));
    }

    #[test]
    fn median_unordered_odd_count() {
        let input = vec![1000_i64, 6000, 2000];
        let avg = TimeStats(&input).get_avg_millis();
        let median = TimeStats(&input).get_median_millis();

        println!("avg={avg}  and median={median}");

        assert!(match_floats(avg, 3.0));
        assert!(match_floats(median, 2.0));
    }

    #[test]
    fn median_unordered_even_count() {
        let input = vec![2000_i64, 6000, 3000, 1000];
        let avg = TimeStats(&input).get_avg_millis();
        let median = TimeStats(&input).get_median_millis();

        println!("avg={avg}  and median={median}");

        assert!(match_floats(avg, 3.0));
        assert!(match_floats(median, 2.5));
    }

    #[test]
    fn median_ordered_even_count_10() {
        let input = vec![
            1000_i64, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000,
        ];
        let avg = TimeStats(&input).get_avg_millis();
        let median = TimeStats(&input).get_median_millis();
        let p90 = TimeStats(&input).get_p_millis(0.9);

        println!("avg={avg}  and median={median}   and p90={p90:?}");

        assert!(match_floats(avg, 5.5));
        assert!(match_floats(median, 5.5));
        assert!(match_float_opts(p90, Some(9.0)));
    }

    #[test]
    fn median_ordered_odd_count_11() {
        let input = vec![
            1000_i64, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 11000,
        ];
        let avg = TimeStats(&input).get_avg_millis();
        let median = TimeStats(&input).get_median_millis();
        let p90 = TimeStats(&input).get_p_millis(0.9);

        println!("avg={avg}  and median={median}   and p90={p90:?}");

        assert!(match_floats(avg, 6.0));
        assert!(match_floats(median, 6.0));
        assert!(match_float_opts(p90, Some(10.0)));
    }
}
