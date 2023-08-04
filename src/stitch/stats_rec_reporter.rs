use crate::stats::json::StatsRecJson;

type SRJProcessor = fn(&StatsRecJson) -> String;

pub struct SRJReportItem {
    label: &'static str,
    processor: SRJProcessor,
}

impl SRJReportItem {
    pub fn new(label: &'static str, processor: SRJProcessor) -> Self {
        Self { label, processor }
    }
}

pub struct StatsRecReporter<'a> {
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRecJson>>,
    report_items: Vec<SRJReportItem>,
}

impl<'a> StatsRecReporter<'a> {
    pub fn new(
        buffer: &'a mut Vec<String>,
        data: &'a Vec<Option<StatsRecJson>>,
        report_items: Vec<SRJReportItem>,
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
            |(idx, SRJReportItem { label, processor })| {
                let values = self
                    .data
                    .iter()
                    .map(|ms| ms.as_ref().map_or("".to_owned(), |srj| processor(srj)))
                    .collect::<Vec<_>>()
                    .join("; ");
                self.buffer.push(format!("{}; {label}; {values}", idx + 1));
            },
        );
    }
}
