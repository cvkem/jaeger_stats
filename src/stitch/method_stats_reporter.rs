use crate::{
    //rate::set_show_rate_output,
    aux::{floats_to_string, format_float_opt, LinearRegression},
    stats::MethodStatsValue,
    stats::StatsRec,
};
use std::collections::HashSet;

use super::{key::Key, stitched::StitchedLine};

/// The POData is the input for the processor (which is a series of report-closures.
/// If the processor operated on a tuple we could extract a joined type from the next two types.
type ProcessorInput<'a> = (&'a MethodStatsValue, i32, usize);
type Processor = fn(&ProcessorInput) -> Option<f64>;
type POData<'a> = Vec<Option<ProcessorInput<'a>>>;

/// Process-Operation report items
pub struct POReportItem {
    label: &'static str,
    processor: Processor,
}

/// this container of ReportItems is primarily used to bundle the methods that runs over a set of .
pub struct POReportItems(pub Vec<POReportItem>);

impl POReportItem {
    pub fn new(label: &'static str, processor: Processor) -> Self {
        Self { label, processor }
    }

    pub fn extract_stitched_line(&self, data: &POData) -> StitchedLine {
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

impl POReportItems {
    /// get all the keys that are relevant for a PO-report
    pub fn get_keys(data: &[Option<StatsRec>]) -> Vec<Key> {
        let mut keys = HashSet::new(); // Can we have duplicates? Is this HashSet needed to deduplicate? However, it does no harm, so leaving it in.
        data.iter().for_each(|stats_rec| {
            if let Some(stats_rec) = stats_rec {
                stats_rec.stats.iter().for_each(|(proc_key, st)| {
                    st.method.0.iter().for_each(|(oper_key, _)| {
                        _ = keys.insert(Key {
                            process: proc_key.to_owned(),
                            operation: oper_key.to_owned(),
                        })
                    })
                })
            }
        });
        let mut keys: Vec<_> = keys.into_iter().collect();
        keys.sort_unstable();
        keys
    }

    pub fn extract_dataset<'a>(data: &'a Vec<Option<StatsRec>>, po_key: &'a Key) -> POData<'a> {
        // a ref to the extract the three values that are needed for the analysis being:
        //    1. the complete MethodStatsValue record
        //    2. the number of files in the analysis
        //    3. the number of traces included in this analysis
        data.iter()
            .map(|stats_rec| {
                stats_rec.as_ref().and_then(|stats_rec| {
                    stats_rec.stats.get(&po_key.process).and_then(|st| {
                        st.method
                            .0
                            .get(&po_key.operation) // can return None!
                            .map(|oper| (oper, stats_rec.num_files, stats_rec.trace_id.len()))
                    })
                })
            })
            .collect()
    }
}

pub struct MethodStatsReporter<'a> {
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRec>>,
    report_items: &'a POReportItems,
}

impl<'a> MethodStatsReporter<'a> {
    pub fn new(
        buffer: &'a mut Vec<String>,
        data: &'a Vec<Option<StatsRec>>,
        report_items: &'a POReportItems,
    ) -> Self {
        Self {
            buffer,
            data,
            report_items,
        }
    }

    // find a deduplicated set of all keys and sort them
    pub fn get_keys(&self) -> Vec<Key> {
        POReportItems::get_keys(self.data)
    }

    pub fn append_report(&mut self, po_key: Key) {
        let po_data = POReportItems::extract_dataset(&self.data, &po_key);

        // do the actual reporting for all files over the selected three values per Method.
        let process_operation = po_key.to_string();
        self.buffer
            .push(format!("# statistics for {process_operation}"));

        self.report_items.0
            .iter()
            .enumerate()
            .for_each(|(idx, po_report_item)| {
                let StitchedLine {
                    label,
                    data,
                    lin_reg,
                } = po_report_item.extract_stitched_line(&po_data);

                // Produce the CSV_output
                let values = floats_to_string(data, "; ");

                let other_columns = 1 + idx * 4;
                let other_columns = (0..other_columns).fold(String::with_capacity(other_columns), |oc, _| oc + ";");
                self.buffer.push(format!(
                    "{process_operation}; {label}; {values}; ; ; {}; {}; {};{other_columns};{}; {}; {};",
                    format_float_opt(lin_reg.slope),
                    format_float_opt(lin_reg.y_intercept),
                    format_float_opt(lin_reg.R_squared),
                    format_float_opt(lin_reg.slope),
                    format_float_opt(lin_reg.y_intercept),
                    format_float_opt(lin_reg.R_squared),
                ));
            });
    }
}
