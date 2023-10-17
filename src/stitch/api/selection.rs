use crate::stitch::stitched::CallChainData;

use super::super::{stitch_list::StitchSources, stitched_set::StitchedSet, Stitched};
use super::{types::SelectLabel, utils, Selection};
use std::{iter, sync::Arc};

impl SelectLabel {
    pub fn new(idx: i64, label: String) -> Self {
        Self {
            idx,
            label,
            selected: true,
        }
    }
}

/// get a numbered and labeled selection with all item selected (the default)
pub fn get_full_selection(data: &Stitched) -> Selection {
    utils::get_label_list(data)
        .into_iter()
        .enumerate()
        .map(|(idx, label)| SelectLabel::new(idx as i64, label))
        .collect()
}

/// get the sources for the current selection, and omit all comments (sources without an index-number)
fn get_stitch_sources(
    original: &Stitched,
    selection: &Vec<bool>,
) -> StitchSources {
    let sources = original.sources.0.iter().filter(|src| src.column.is_some());
    let sel_src = iter::zip(selection, sources)
        .filter_map(|(sel, src)| if *sel { Some((*src).clone()) } else { None })
        .collect();
    StitchSources( sel_src )
}

/// get a copy of the process_operation data for a specific selection
fn get_proc_oper_selection(
    original: &Stitched,
    selection: &Vec<bool>,
) -> Vec<(String, StitchedSet)> {
    original
        .process_operation
        .iter()
        .filter_map(
            |(k, stitched_set)| match stitched_set.get_selection(selection) {
                Some(selection) => Some((k.to_owned(), selection)),
                None => None,
            },
        )
        .collect()
}

/// get a copy of the call_chain data for a specific selection
fn get_call_chain_selection(
    original: &Stitched,
    selection: &Vec<bool>,
) -> Vec<(String, Vec<CallChainData>)> {
    original
        .call_chain
        .iter()
        .filter_map(|(k, ccd_vec)| {
            let data: Vec<_> = ccd_vec
                .iter()
                .filter_map(|ccd| ccd.get_selection(selection))
                .collect();
            if !data.is_empty() {
                Some((k.to_owned(), data))
            } else {
                None
            }
        })
        .collect()
}

/// get a derived dataset that only contains the selected columns
pub fn get_derived_stitched(original: &Stitched, selection: &Vec<bool>) -> Arc<Stitched> {
    let process_operation = get_proc_oper_selection(original, selection);
    let call_chain = get_call_chain_selection(original, selection);
    let sources = get_stitch_sources(original, selection);
    Arc::new(Stitched {
        sources,
        basic: StitchedSet(Vec::new()),     // exluded from copy
        process_operation,
        call_chain,
    })
}
