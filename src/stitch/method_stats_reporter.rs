use crate::{
    //rate::set_show_rate_output,
    stats::MethodStatsValue,
    stats::StatsRec,
};
use std::collections::HashSet;

use super::key::Key;

type Processor = fn(&MethodStatsValue, i32, usize) -> String;

pub struct MSReportItem {
    label: &'static str,
    processor: Processor,
}

impl MSReportItem {
    pub fn new(label: &'static str, processor: Processor) -> Self {
        Self { label, processor }
    }
}

pub struct MethodStatsReporter<'a> {
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRec>>,
    report_items: Vec<MSReportItem>,
}

impl<'a> MethodStatsReporter<'a> {
    pub fn new(
        buffer: &'a mut Vec<String>,
        data: &'a Vec<Option<StatsRec>>,
        report_items: Vec<MSReportItem>,
    ) -> Self {
        Self {
            buffer,
            data,
            report_items,
        }
    }

    // find a deduplicated set of all keys and sort them
    pub fn get_keys(&self) -> Vec<Key> {
        let mut keys = HashSet::new(); // Can we have duplicates? Is this HashSet needed to deduplicate? However, it does no harm, so leaving it in.
        self.data.iter().for_each(|stats_rec_json| {
            if let Some(stats_rec_json) = stats_rec_json {
                stats_rec_json.stats.iter().for_each(|(proc_key, st)| {
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
        keys.sort();
        keys
    }

    pub fn append_report(&mut self, process: String, operation: String) {
        let meth_stats: Vec<_> = self
            .data
            .iter()
            .map(|stats_rec_json| match stats_rec_json {
                Some(stats_rec_json) => match stats_rec_json.stats.get(&process) {
                    Some(st) => match st.method.0.get(&operation) {
                        Some(oper) => Some((
                            oper,
                            stats_rec_json.num_files,
                            stats_rec_json.trace_id.len(),
                        )),
                        None => None,
                    },
                    None => None,
                },
                None => None,
            })
            .collect();

        let process_operation = format!("{process}/{operation}");
        self.buffer
            .push(format!("# statistics for {process_operation}"));

        // set_show_rate_output(&process_operation[..] == "bspc-productinzicht/GET");

        self.report_items
            .iter()
            .for_each(|MSReportItem { label, processor }| {
                let values = meth_stats
                    .iter()
                    .map(|ms| {
                        ms.map_or("".to_owned(), |msv_nf| {
                            processor(msv_nf.0, msv_nf.1, msv_nf.2)
                        })
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                self.buffer
                    .push(format!("{process_operation}; {label}; {values}"));
            });
    }
}
