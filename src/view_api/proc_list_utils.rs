use super::types::ProcessList;
use std::cmp::Ordering;

/// Reorder the list of processes based on the rank field when it has metric, otherwise sort lexicographic on name.
pub fn reorder_and_renumber(proc_list: ProcessList, has_metric: bool) -> ProcessList {
    if has_metric {
        rank_reorder_and_renumber(proc_list)
    } else {
        lexicographic_reorder(proc_list)
    }
}

/// Reorder on the key.
/// TODO: reorder on display-name first and then on key
fn lexicographic_reorder(mut proc_list: ProcessList) -> ProcessList {
    proc_list.sort_by(|a, b| a.key.cmp(&b.key));
    let list_len = proc_list.len();

    // renumber for new ordering
    proc_list.iter_mut().enumerate().for_each(|(idx, pli)| {
        pli.idx = (idx + 1) as i64;
        pli.rank = (list_len - idx) as f64;
    });

    proc_list
}

/// Reorder the list of processes based on the rank field and renumber if the 'metric' is set.
fn rank_reorder_and_renumber(mut proc_list: ProcessList) -> ProcessList {
    proc_list.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(Ordering::Equal));

    // renumber for new ordering
    proc_list
        .iter_mut()
        .enumerate()
        .for_each(|(idx, pli)| pli.idx = (idx + 1) as i64);
    proc_list
}
