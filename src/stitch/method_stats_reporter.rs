use std::collections::HashSet;
use crate::{
    stats_json::StatsRecJson, 
    method_stats::MethodStatsValue};


use super::key::Key;


pub struct ReportItem {
    label: &'static str,
    processor: fn(&MethodStatsValue, i32) -> String,
}

impl ReportItem {
    pub fn new (label: &'static str, processor: fn(&MethodStatsValue, i32) -> String) -> Self {
        Self{label, processor}
    }
}

pub struct MethodStatsReporter<'a>{
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRecJson>>,
    report_items: Vec<ReportItem>
}

impl<'a> MethodStatsReporter<'a> {

    pub fn new(buffer: &'a mut Vec<String>, data: &'a Vec<Option<StatsRecJson>>, report_items: Vec<ReportItem>) -> Self {
        // find a deduplicated set of all keys and sort them 

        Self{buffer, data, report_items}
    }

    pub fn get_keys(&self) -> Vec<Key> {
        let mut keys  = HashSet::new();
        self.data.iter()
            .for_each(|str| {
                if let Some(str) = str {
                    str.stats.iter()
                        .for_each(|(proc_key, st)| {
                            st.method.0.iter()
                                .for_each(|(oper_key, _)| _ = keys.insert(Key{process: proc_key.to_owned(), operation: oper_key.to_owned()}))
                        })
                }
            });
        let mut keys: Vec<_> = keys.into_iter().collect();
        keys.sort();
        keys
    }


    pub fn append_report(&mut self, process: String, operation: String) {
        let meth_stats: Vec<_> = self.data.iter()
        .map(|str| {
            match str {
                Some(str) => {
                    match str.stats.get(&process) {
                        Some(st) => match st.method.0.get(&operation) {
                            Some(oper) => Some((oper, str.num_files.unwrap_or(0))),
                            None => None
                        },
                        None => None
                    }
                }
                None => None
            }
        })
        .collect();

        let process_operation = format!("{process}/{operation}");
        self.buffer.push(format!("# statistics for {process_operation}"));

        self.report_items
            .iter()
            .for_each(|ReportItem{label, processor}| {
                let values = meth_stats.iter()
                    .map(|ms| ms.map_or("".to_owned(),|msv_nf |processor(msv_nf.0, msv_nf.1)))
                    .collect::<Vec<_>>()
                    .join("; ");
                self.buffer.push(format!("{process_operation}; {label}; {values}"));    
            });

    }
}

