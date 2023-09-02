use super::stitched_line::StitchedLine;
use crate::{
    stats::call_chain::CChainStatsKey,
    stats::{call_chain::CChainStatsValue, StatsRec},
    AnomalyParameters,
};

use std::{cmp::Ordering, collections::HashMap};

/// The POData is the input for the processor (which is a series of report-closures.
/// If the processor operated on a tuple we could extract a joined type from the next two types.
type ProcessorInput<'a> = (&'a CChainStatsValue, i32, usize);
type Processor = fn(&ProcessorInput) -> Option<f64>;
type CCData<'a> = Vec<Option<ProcessorInput<'a>>>;

/// Call-chain report items are defined in this structure.
/// TODO: as this is a copy of the POReportItem, including all code we should move this to generics
///   only the Processor-type and thus the input data are different.
pub struct CCReportItem {
    pub label: &'static str,
    processor: Processor,
}

// this container of ReportItems is primarily used to bundle the methods that runs over a set of .
/// TODO: when using generics we could have one-codebase for POReportItems and CCReportItems
pub struct CCReportItems(pub Vec<CCReportItem>);

impl CCReportItem {
    pub fn new(label: &'static str, processor: Processor) -> Self {
        Self { label, processor }
    }

    pub fn extract_stitched_line(&self, data: &CCData, pars: &AnomalyParameters) -> StitchedLine {
        let values = data
            .iter()
            .map(|ms| ms.as_ref().and_then(self.processor))
            .collect::<Vec<_>>();

        StitchedLine::new(self.label.to_string(), values, pars)
    }
}

type CCKey = (String, Vec<CChainStatsKey>);

impl CCReportItems {
    /// repartition the keys by grouping on the string value (proc_oper) and then on the usize decending
    /// the resturned value groups the cck-keys behind each proc_oper in decending count order (So most frequent call-chains on top of the table)
    fn repartition_keys(mut keys: Vec<(String, CChainStatsKey, usize)>) -> Vec<CCKey> {
        keys.sort_unstable_by(|a, b| match a.0.cmp(&b.0) {
            Ordering::Equal => a.2.cmp(&b.2).reverse(),
            other => other,
        });

        keys.into_iter()
            .fold(
                (Vec::<CCKey>::new(), String::new()),
                |(mut acc, curr_po), (proc_oper, cck, _)| {
                    if proc_oper == curr_po {
                        //acc[acc.len()-1].1.push(cck);   // needs to be split out to two lines to satisfy borrow-checker
                        let len = acc.len();
                        acc[len - 1].1.push(cck);
                        (acc, curr_po)
                    } else {
                        let curr_po = proc_oper.to_owned();
                        acc.push((proc_oper.to_owned(), [cck].to_vec()));
                        (acc, curr_po)
                    }
                },
            )
            .0
    }

    /// get all the keys that are relevant for a CC-report
    /// TODO: when using generics we could have one-codebase for POReportItems and CCReportItems
    /// Here it is only the inner loop that generates a key that needs to be split out.
    pub fn get_keys(data: &[Option<StatsRec>]) -> Vec<CCKey> {
        let mut keys = HashMap::new(); // Computing all possible keys over the different datasets that need to be stitched.
        data.iter().for_each(|stats_rec| {
            if let Some(stats_rec) = stats_rec {
                stats_rec.stats.iter().for_each(|(proc_key, st)| {
                    st.call_chain.iter().for_each(|(cc_key, cc_val)| {
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
                        keys.entry(cc_key_clone)
                            .and_modify(|v| *v += cc_val.count)
                            .or_insert(cc_val.count);
                    })
                })
            }
        });
        let keys: Vec<_> = keys
            .into_iter()
            .map(|(cck, count)| (cck.get_leaf(), cck, count))
            .collect();

        // make grouping based on the Process_operation field (get_leaf)
        Self::repartition_keys(keys)
    }

    pub fn extract_dataset<'a>(
        data: &'a [Option<StatsRec>],
        cc_key: &'a CChainStatsKey,
    ) -> CCData<'a> {
        // We only need to search in the subset that belongs to this process as that pare will contain all records for this call-chain.
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
