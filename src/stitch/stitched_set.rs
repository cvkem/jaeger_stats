use crate::Metric;

use super::{anomalies::DEFAULT_ANOMALY_PARS, stitched_line::StitchedLine};
use std::iter;

use serde::{Deserialize, Serialize};

/// A StitchedSet is a vector of StitchedLine items.
/// Each StitchedLine represents a metric and carries a label that represents that data. The contents of the StitchedLine is a time-series of the data and a linear-regression over that data.
#[derive(Default, Debug, Deserialize, Serialize)]
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
        let headers = self.0.iter().map(|sl| sl.metric.to_string());
        if extra_count {
            let count_label = self
                .0
                .first()
                .map(|sl| sl.metric.to_string())
                .unwrap_or("NO DATA".to_owned());
            [count_label, "NUM_FILLED".to_string()]
                .iter()
                .cloned()
                .chain(headers)
                .collect()
        } else {
            headers.collect()
        }
    }

    pub fn summary_avg(&self) -> Vec<Option<f64>> {
        self.0.iter().map(|sl| sl.data_avg).collect()
    }

    /// given an iterator of Option<f64> it is prefixed with two values,
    fn prefix_with_counts(&self, data: impl Iterator<Item = Option<f64>>) -> Vec<Option<f64>> {
        // NOTE: here we assume first line always is a count-metric
        let count = self.0.first().and_then(|data| data.data_avg);
        let num_filled_columns = self.0.first().map(|data| data.num_filled_columns as f64);
        let prefix = [count, num_filled_columns].to_vec();
        prefix.into_iter().chain(data).collect()
    }

    pub fn summary_slopes(&self) -> Vec<Option<f64>> {
        self.prefix_with_counts(
            self.0
                .iter()
                .map(|sl| sl.lin_regr.as_ref().map(|lr| lr.slope)),
        )
    }

    pub fn summary_last_deviation_scaled(&self) -> Vec<Option<f64>> {
        self.prefix_with_counts(self.0.iter().map(|sl| sl.last_deviation_scaled()))
    }

    pub fn summary_scaled_slopes(&self) -> Vec<Option<f64>> {
        self.prefix_with_counts(self.0.iter().map(|sl| sl.scaled_slope()))
    }

    pub fn get_metric_stitched_line(&self, metric: Metric) -> Option<&StitchedLine> {
        self.0.iter().find(|line| line.metric == metric)
    }

    /// Get a subset of selected data-points for each of the stiched lines in the stitched set, or None if the selection does not contain any f64 values (only None)
    /// assume that the size of the selection was checked by the upstream process (the caller).
    pub fn get_selection(&self, selection: &[bool]) -> Option<Self> {
        let data: Vec<_> = self
            .0
            .iter()
            .map(|sl| {
                let selected: Vec<_> = iter::zip(selection.iter(), sl.data.iter())
                    .filter_map(|(sel, val)| if *sel { Some(*val) } else { None })
                    .collect();
                let has_value = selected.iter().any(|x| x.is_some());
                (&sl.metric, selected, has_value)
            })
            .collect();

        if data.iter().any(|(_, _, has_value)| *has_value) {
            Some(StitchedSet(
                data.into_iter()
                    .map(|(lbl, data, _)| {
                        StitchedLine::compute_new(lbl.to_owned(), data, &DEFAULT_ANOMALY_PARS)
                    })
                    .collect(),
            ))
        } else {
            None
        }
    }
}
