//! Deduplication of traces based on the GUID (Identifier of the traces)
use crate::{
    aux::{report, Chapter},
    processed::Trace,
    stats::TraceExt,
};
use std::collections::HashSet;

/// deduplicate all the traces based on traceId and report effect,
pub fn deduplicate(traces: Vec<Trace>) -> Vec<Trace> {
    let initial_num = traces.len();

    let mut observed_id = HashSet::new();
    let mut duplicated_ids = Vec::new();

    let traces: Vec<_> = traces
        .into_iter()
        .filter(|tr| {
            let trace_id = &tr.trace_id;
            if observed_id.insert(trace_id.clone()) {
                true // this is a new trace_id
            } else {
                duplicated_ids.push(trace_id.clone());
                false
            }
        })
        .collect();

    let num_duplicates = duplicated_ids.len();
    let remaining = traces.len();
    report(
        Chapter::Summary,
        format!(
            "Removed {num_duplicates}:  So list of {initial_num} traces reduced to {remaining}"
        ),
    );
    report(
        Chapter::Details,
        format!("Removed duplicates: {duplicated_ids:?}"),
    );

    traces
}

// /// deduplicate all the traces based on traceId and report effect,
// pub fn deduplicate_trace_ext(traces: Vec<TraceExt>) -> Vec<TraceExt> {
//     let initial_num = traces.len();

//     let mut observed_id = HashSet::new();
//     let mut duplicated_ids = Vec::new();

//     let traces: Vec<_> = traces
//         .into_iter()
//         .filter(|tr| {
//             let trace_id = &tr.trace.trace_id;
//             if observed_id.insert(trace_id.clone()) {
//                 true // this is a new trace_id
//             } else {
//                 duplicated_ids.push(trace_id.clone());
//                 false
//             }
//         })
//         .collect();

//     let num_duplicates = duplicated_ids.len();
//     let remaining = traces.len();
//     report(
//         Chapter::Summary,
//         format!(
//             "Removed {num_duplicates}:  So list of {initial_num} traces reduced to {remaining}"
//         ),
//     );
//     report(
//         Chapter::Details,
//         format!("Removed duplicates: {duplicated_ids:?}"),
//     );

//     traces
// }
