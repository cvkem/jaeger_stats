use std::collections::HashMap;
use crate::{Spans};




/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn basic_stats(spans: &Spans) -> HashMap<String, u32> {
    let mut stats = HashMap::new();
    spans
        .iter()
        .for_each(|span| {
            let proc = match &span.process {
                Some(p) => &p.name[..],
                None => "-"
            };
            let proc_method = format!("{}/{}", proc, span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}


fn get_parent_processes(parent_idx: Option<usize>, spans: &Spans) -> String {
    let mut parents = Vec::new();
    let mut par_idx = parent_idx;
    loop {
        match par_idx {
            None => break,
            Some(idx) => {
                let par_span = &spans[idx];
                parents.push(match &par_span.process {
                    Some(p) => p.name.to_owned(),
                    None => "-".to_owned()
                });
                par_idx = par_span.parent;
            }
        }
    }
    let res = parents
        .into_iter()
        .rev()
        .fold(String::new(), |a, b| a + &b + "|");
    res
}


/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn chained_stats(spans: &Spans) -> HashMap<String, u32> {
    let mut stats = HashMap::new();
    spans
        .iter()
        .for_each(|span| {
            let proc = match &span.process {
                Some(p) => &p.name[..],
                None => "-"
            };
            let parents_str = get_parent_processes(span.parent, spans);
            let proc_method = format!("{parents_str}{proc}/{}", span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}

