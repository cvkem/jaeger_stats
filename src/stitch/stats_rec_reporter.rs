use super::stitched_line::StitchedLine;
use crate::{stats::StatsRec, AnomalyParameters, Metric};

type SRProcessor = fn(&StatsRec) -> Option<f64>;

pub struct SRReportItem {
    metric: Metric,
    processor: SRProcessor,
}

impl SRReportItem {
    pub fn new(metric: Metric, processor: SRProcessor) -> Self {
        Self { metric, processor }
    }

    /// extract values
    pub fn extract_data(&self, data: &[Option<StatsRec>]) -> Vec<Option<f64>> {
        data.iter()
            .map(|ms| ms.as_ref().and_then(self.processor))
            .collect::<Vec<_>>()
    }

    /// extract a line of stitched data for the current report item.
    pub fn extract_stitched_line(
        &self,
        data: &[Option<StatsRec>],
        pars: &AnomalyParameters,
    ) -> StitchedLine {
        let values = self.extract_data(data);

        StitchedLine::new(self.metric, values, pars)
    }
}
