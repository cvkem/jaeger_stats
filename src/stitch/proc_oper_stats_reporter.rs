use crate::{stats::ProcOperStatsValue, stats::StatsRec, AnomalyParameters};
use std::collections::HashSet;

use super::{key::Key, stitched_line::StitchedLine};

/// The POData is the input for the processor (which is a series of report-closures.
/// If the processor operated on a tuple we could extract a joined type from the next two types.
type ProcessorInput<'a> = (&'a ProcOperStatsValue, i32, usize);
type Processor = fn(&ProcessorInput) -> Option<f64>;
type POData<'a> = Vec<Option<ProcessorInput<'a>>>;

/// Process-Operation report items
pub struct POReportItem {
    pub label: &'static str,
    processor: Processor,
}

/// this container of ReportItems is primarily used to bundle the methods that runs over a set of .
pub struct POReportItems(pub Vec<POReportItem>);

impl POReportItem {
    pub fn new(label: &'static str, processor: Processor) -> Self {
        Self { label, processor }
    }

    pub fn extract_stitched_line(&self, data: &POData, pars: &AnomalyParameters) -> StitchedLine {
        let values = data
            .iter()
            .map(|ms| ms.as_ref().and_then(self.processor))
            .collect::<Vec<_>>();

        StitchedLine::new(self.label.to_string(), values, pars)
    }
}

impl POReportItems {
    /// get all the keys that are relevant for a PO-report
    pub fn get_keys(data: &[Option<StatsRec>]) -> Vec<Key> {
        let mut keys = HashSet::new(); // Can we have duplicates? Is this HashSet needed to deduplicate? However, it does no harm, so leaving it in.
        data.iter().for_each(|stats_rec| {
            if let Some(stats_rec) = stats_rec {
                stats_rec.stats.iter().for_each(|(proc_key, st)| {
                    st.operation.0.iter().for_each(|(oper_key, _)| {
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
                        st.operation
                            .0
                            .get(&po_key.operation) // can return None!
                            .map(|oper| (oper, stats_rec.num_files, stats_rec.trace_id.len()))
                    })
                })
            })
            .collect()
    }
}
