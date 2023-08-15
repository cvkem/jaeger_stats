use super::stitched_set::StitchedLine;
use crate::{
    aux::{floats_to_string, format_float_opt, LinearRegression},
    stats::call_chain::CChainStatsKey,
    stats::{call_chain::CChainStatsValue, StatsRec},
};

use std::collections::HashSet;

/// The POData is the input for the processor (which is a series of report-closures.
/// If the processor operated on a tuple we could extract a joined type from the next two types.
type ProcessorInput<'a> = (&'a CChainStatsValue, i32, usize);
type Processor = fn(&ProcessorInput) -> Option<f64>;
type CCData<'a> = Vec<Option<ProcessorInput<'a>>>;

/// Call-chain report items are defined in this structure.
/// TODO: as this is a copy of the POReportItem, including all code we should move this to generics
///   only the Processor-type and thus the input data are different.
pub struct CCReportItem {
    label: &'static str,
    processor: Processor,
}

// this container of ReportItems is primarily used to bundle the methods that runs over a set of .
/// TODO: when using generics we could have one-codebase for POReportItems and CCReportItems
pub struct CCReportItems(pub Vec<CCReportItem>);

impl CCReportItem {
    pub fn new(label: &'static str, processor: Processor) -> Self {
        Self { label, processor }
    }

    pub fn extract_stitched_line(&self, data: &CCData) -> StitchedLine {
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

pub struct CallChainReporter<'a> {
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRec>>,
    report_items: &'a CCReportItems,
}

impl CCReportItems {
    /// get all the keys that are relevant for a CC-report
    /// TODO: when using generics we could have one-codebase for POReportItems and CCReportItems
    /// Here it is only the inner loop that generates a key that needs to be split out.
    pub fn get_keys(data: &[Option<StatsRec>]) -> Vec<CChainStatsKey> {
        let mut keys = HashSet::new(); // Computing all possible keys over the different datasets that need to be stitched.
        data.iter().for_each(|stats_rec| {
            if let Some(stats_rec) = stats_rec {
                stats_rec.stats.iter().for_each(|(proc_key, st)| {
                    st.call_chain.iter().for_each(|(cc_key, _)| {
                        // checks
                        let cc_key_clone = cc_key.clone();
                        if *cc_key != cc_key_clone {
                            println!("Failed to clone for '{cc_key:#?}'.")
                        };
                        let process = cc_key_clone.get_leaf_process();
                        if *proc_key != process {
                            println!(
                                "Mismatch between '{proc_key}' and extracted proces '{process}'"
                            )
                        }

                        _ = keys.insert(cc_key_clone)
                    })
                })
            }
        });
        let mut keys: Vec<_> = keys.into_iter().collect();
        keys.sort_unstable();
        keys
    }

    pub fn extract_dataset<'a>(
        data: &'a [Option<StatsRec>],
        cc_key: &'a CChainStatsKey,
    ) -> CCData<'a> {
        let process = cc_key.get_leaf_process();

        // a ref to the extract the three values that are needed for the analysis being:
        //    1. the complete CallChainValue record
        //    2. the number of files in the analysis
        //    3. the number of traces included in this analysis
        data.iter()
            .map(|stats_rec| {
                stats_rec.as_ref().and_then(|stats_rec| {
                    stats_rec.stats.get(&process).and_then(|st| {
                        st.call_chain
                            .get(cc_key)
                            .map(|oper| (oper, stats_rec.num_files, stats_rec.trace_id.len()))
                    })
                })
            })
            .collect()
    }
}

impl<'a> CallChainReporter<'a> {
    pub fn new(
        buffer: &'a mut Vec<String>,
        data: &'a Vec<Option<StatsRec>>,
        report_items: &'a CCReportItems,
    ) -> Self {
        Self {
            buffer,
            data,
            report_items,
        }
    }

    // find a deduplicated set of all keys and sort them
    pub fn get_keys(&self) -> Vec<CChainStatsKey> {
        CCReportItems::get_keys(self.data)
    }

    pub fn append_report(&mut self, cc_key: CChainStatsKey) {
        let cc_data = CCReportItems::extract_dataset(&self.data, &cc_key);

        let cc_key_str = cc_key.to_string();
        self.buffer.push(format!("# statistics for {cc_key_str}"));

        // set_show_rate_output(&process_operation[..] == "bspc-productinzicht/GET");

        self.report_items
            .0
            .iter()
            .enumerate()
            .for_each(|(idx, cc_report_item)| {
                let StitchedLine {
                    label,
                    data,
                    lin_reg,
                } = cc_report_item.extract_stitched_line(&cc_data);

                // Produce the CSV_output
                let values = floats_to_string(data, "; ");

                let other_columns = 1 + idx * 4;
                let other_columns =
                    (0..other_columns).fold(String::with_capacity(other_columns), |oc, _| oc + ";");
                self.buffer.push(format!(
                    "{cc_key_str}; {label}; {values}; ; ; {}; {}; {};{other_columns};{}; {}; {};",
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
