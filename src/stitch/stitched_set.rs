use serde_json::map::Iter;

use super::stitched_line::StitchedLine;
use std::iter;

/// A StitchedSet is a vector of StitchedLine items.
/// Each StitchedLine represents a metric and carries a label that represents that data. The contents of the StitchedLine is a time-series of the data and a linear-regression over that data.
#[derive(Default, Debug)]
pub struct StitchedSet(pub Vec<StitchedLine>);

impl StitchedSet {
    pub fn csv_output(&self, prefixes: &[&str]) -> Vec<String> {
        self.0
            .iter()
            .map(|line| line.to_csv_string(prefixes))
            .collect()
    }

    /// Generates a header-line to be shown before the summary of statistics based on the labels of the metrics present in this StitchedSet, where a single statistic is computed for each of the metrics.
    /// If 'extra_count' is true than an additional Count-column is added.
    /// On the normal summary that shows the averages per metric 'extra_count' is set to false as 'Count' is the first metric.
    /// However, in summary reports that use another statistic, such as 'slope' or R2 value, the 'extra_count' is set to true as the existing Count column in that case does not represents the average.  
    pub fn summary_header(&self, extra_count: bool) -> Vec<String> {
        let headers = self.0.iter().map(|sl| sl.label.to_owned());
        if extra_count {
            let count_label = self
                .0
                .first()
                .map(|sl| sl.label.to_uppercase())
                .unwrap_or("NO DATA".to_owned());
            [count_label, "NUM_FILLED".to_string()]
                .to_vec()
                .into_iter()
                .chain(headers)
                .collect()
        } else {
            headers.collect()
        }
    }

    /// Header for the full data-report over a single metric (a specific StitchedLine) in this StitchedSet
    /// A StitchedLine is a time-series of the data and a linear-regression over that data, so this header contains a column for each time-slice and a few extra columns for the lineair regression.
    pub fn full_data_header(&self) -> String {
        if self.0.is_empty() {
            "NO DATA".to_owned()
        } else {
            self.0[0].headers()
        }
    }

    pub fn summary_avg(&self) -> Vec<Option<f64>> {
        self.0.iter().map(|sl| sl.data_avg).collect()
    }

    fn prefix_with_counts(&self, data: impl Iterator<Item = Option<f64>>) -> Vec<Option<f64>> {
        // NOTE: here we assume first line always is a count-metric
        let count = self.0.first().and_then(|data| data.data_avg);
        let num_filled_columns = self
            .0
            .first()
            .and_then(|data| Some(data.num_filled_columns as f64));
        let prefix = [count, num_filled_columns].to_vec();
        prefix.into_iter().chain(data).collect()
    }

    pub fn summary_slopes(&self) -> Vec<Option<f64>> {
        self.prefix_with_counts(
            self.0
                .iter()
                .map(|sl| sl.lin_reg.as_ref().map(|lr| lr.slope)),
        )
    }

    pub fn summary_last_deviation_scaled(&self) -> Vec<Option<f64>> {
        self.prefix_with_counts(self.0.iter().map(|sl| sl.last_deviation_scaled()))
    }

    pub fn summary_scaled_slopes(&self) -> Vec<Option<f64>> {
        self.prefix_with_counts(self.0.iter().map(|sl| sl.scaled_slope()))
    }
}
