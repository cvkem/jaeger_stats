use super::stitched::StitchedLine;
use crate::{
    aux::{floats_to_string, format_float_opt, LinearRegression},
    stats::StatsRec,
};

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
    pub fn extract_stitched_line(&self, data: &Vec<Option<StatsRec>>) -> StitchedLine {
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

///TODO The remainder of this document is legacy code to be discarded after CSV-output has been extracted

pub struct StatsRecReporterCSV<'a> {
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRec>>,
    report_items: &'a Vec<SRReportItem>,
}

impl<'a> StatsRecReporterCSV<'a> {
    pub fn new(
        buffer: &'a mut Vec<String>,
        data: &'a Vec<Option<StatsRec>>,
        report_items: &'a Vec<SRReportItem>,
    ) -> Self {
        // find a deduplicated set of all keys and sort them

        Self {
            buffer,
            data,
            report_items,
        }
    }

    pub fn append_report(&mut self) {
        self.report_items
            .iter()
            .enumerate()
            .for_each(|(idx, sr_report)| {
                let StitchedLine {
                    label,
                    data,
                    lin_reg,
                } = sr_report.extract_stitched_line(&self.data);

                let values = floats_to_string(data, "; ");

                let other_columns = 1 + idx * 4;
                let other_columns =
                    (0..other_columns).fold(String::with_capacity(other_columns), |oc, _| oc + ";");
                self.buffer.push(format!(
                    "{label}; {values}; ; ; {}; {}; {};{other_columns};{}; {}; {};",
                    format_float_opt(lin_reg.slope),
                    format_float_opt(lin_reg.y_intercept),
                    format_float_opt(lin_reg.R_squared),
                    // get data again, but now separated in columns per reportItem
                    format_float_opt(lin_reg.slope),
                    format_float_opt(lin_reg.y_intercept),
                    format_float_opt(lin_reg.R_squared),
                ));
            });
    }
}
