use super::stitched_set::StitchedLine;
use crate::{stats::StatsRec, utils::LinearRegression};

type SRProcessor = fn(&StatsRec) -> Option<f64>;

pub struct SRReportItem {
    label: &'static str,
    processor: SRProcessor,
}

impl SRReportItem {
    pub fn new(label: &'static str, processor: SRProcessor) -> Self {
        Self { label, processor }
    }

    /// extract a line of stitched data for the current report item.
    pub fn extract_stitched_line(&self, data: &[Option<StatsRec>]) -> StitchedLine {
        let values = data
            .iter()
            .map(|ms| ms.as_ref().and_then(self.processor))
            .collect::<Vec<_>>();

        let lin_reg = LinearRegression::new(&values);
        StitchedLine {
            label: self.label.to_string(),
            data: values,
            lin_reg,
        }
    }
}
