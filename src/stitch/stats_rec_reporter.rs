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
}

pub struct StatsRecReporter<'a> {
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRec>>,
    report_items: Vec<SRReportItem>,
}

impl<'a> StatsRecReporter<'a> {
    pub fn new(
        buffer: &'a mut Vec<String>,
        data: &'a Vec<Option<StatsRec>>,
        report_items: Vec<SRReportItem>,
    ) -> Self {
        // find a deduplicated set of all keys and sort them

        Self {
            buffer,
            data,
            report_items,
        }
    }

    pub fn append_report(&mut self) {
        self.report_items.iter().enumerate().for_each(
            |(idx, SRReportItem { label, processor })| {
                let values = self
                    .data
                    .iter()
                    .map(|ms| ms.as_ref().and_then(processor))
                    .collect::<Vec<_>>();

                let lr = LinearRegression::new(&values);

                let values = floats_to_string(values, "; ");

                let other_columns = 1 + idx * 4;
                let other_columns =
                    (0..other_columns).fold(String::with_capacity(other_columns), |oc, _| oc + ";");
                self.buffer.push(format!(
                    "{label}; {values}; ; ; {}; {}; {};{other_columns};{}; {}; {};",
                    format_float_opt(lr.slope),
                    format_float_opt(lr.y_intercept),
                    format_float_opt(lr.R_squared),
                    format_float_opt(lr.slope),
                    format_float_opt(lr.y_intercept),
                    format_float_opt(lr.R_squared),
                ));
            },
        );
    }
}
