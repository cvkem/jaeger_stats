use super::super::Stitched;
use super::selection::get_derived_stitched;
use super::{
    selection::get_full_selection,
    types::{ChartDataParameters, ProcessList, Selection, Table},
    utils, TraceScope,
};
use crate::mermaid;
use log::{error, info};
use std::{path::Path, sync::Arc};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // correct propagation requires a clean-up of upstream error-handling
    // example: https://kerkour.com/rust-error-handling
    // #[error("Failed loading file")]
    // load_failure(#[from] dyn error::Error),
    #[error("Failed loading file {0} with error {1}")]
    load_failure(String, String),

    #[error("the file `{0}` is not found")]
    does_not_exist(String),

    #[error("Mismatch in length of the selection which contains {0} elements, while the original dataset has {1} columns.")]
    selection_failure(usize, usize),
}

pub struct StitchedDataSet {
    /// current dataset used for most of the operations
    current: Arc<Stitched>,
    /// The original dataset in case we are working on a selection of the original data, or None if current is the original dataset
    original: Arc<Stitched>,

    data_selection: Selection,
}

impl StitchedDataSet {
    pub fn new(data: Stitched) -> Self {
        let data_selection = get_full_selection(&data);
        let original = Arc::new(data);
        let current = original.clone();
        Self {
            current,
            original,
            data_selection,
        }
    }

    pub fn from_file(file_name: &str) -> Result<Self, Error> {
        if Path::new(file_name).exists() {
            info!("Trying to load the file {file_name}");

            match Stitched::from_json(file_name) {
                Ok(data) => {
                    info!("Ready loading file");
                    Ok(Self::new(data))
                }
                Err(err) => Err(Error::load_failure(
                    file_name.to_owned(),
                    format!("{err:?}"),
                )),
            }
        } else {
            let msg = format!("ERROR: File '{file_name} does not exist");
            error!("{msg}");
            Err(Error::does_not_exist(file_name.to_owned()))
        }
    }

    /// Get a copy of the cached label-list
    fn get_label_list(&self) -> Vec<String> {
        self.data_selection
            .iter()
            .filter(|label_item| label_item.selected)
            .map(|label_item| label_item.label.to_owned())
            .collect()
    }

    pub fn get_process_list(&self, metric: &str) -> ProcessList {
        utils::get_process_list(&self.current, metric)
    }

    pub fn get_call_chain_list(
        &self,
        proc_oper: &str,
        metric: &str,
        scope: TraceScope,
        inbound_idx: Option<i64>,
    ) -> ProcessList {
        utils::get_call_chain_list(&self.current, proc_oper, metric, scope, inbound_idx)
    }

    pub fn get_proc_oper_chart_data(
        &self,
        process: &str,
        metric: &str,
    ) -> Option<ChartDataParameters> {
        utils::get_service_oper_chart_data(&self.current, self.get_label_list(), process, metric)
    }

    /// get a mermaid diagram that depicts the current selection based on proc_oper and optionally a call-chain.
    pub fn get_mermaid_diagram(
        &self,
        proc_oper: &str,
        call_chain_key: Option<&str>,
        scope: mermaid::MermaidScope,
        compact: bool,
    ) -> String {
        let trace_tree = self
            .current
            .call_chain
            .iter()
            .map(|(k, ccd)| {
                let trace_data = ccd
                    .iter()
                    .map(|ccd| {
                        let count: u64 = ccd
                            .data
                            .0
                            .first()
                            .and_then(|data| data.data_avg)
                            .unwrap()
                            .round() as u64;
                        let avg_duration_millis = ccd
                            .data
                            .0
                            .iter()
                            .filter(|x| x.label == "avg_duration_millis")
                            .next()
                            .and_then(|data| data.data_avg)
                            .expect("avg-duration missing");
                        mermaid::TraceData::new(
                            &ccd.full_key,
                            ccd.rooted,
                            ccd.is_leaf,
                            count,
                            avg_duration_millis,
                            None,
                            None,
                            None,
                            None,
                        )
                    })
                    .collect();
                (k.clone(), trace_data)
            })
            .collect();
        mermaid::TracePaths(trace_tree).get_diagram(
            proc_oper,
            call_chain_key,
            crate::EdgeValue::Count,
            scope,
            compact,
        )
    }

    pub fn get_call_chain_chart_data(
        &self,
        call_chain_key: &str,
        metric: &str,
    ) -> Option<ChartDataParameters> {
        utils::get_call_chain_chart_data(
            &self.current,
            self.get_label_list(),
            call_chain_key,
            metric,
        )
    }

    /// filestats are always derived from the original dataset
    pub fn get_file_stats(&self) -> Table {
        utils::get_file_stats(&self.original)
    }

    pub fn get_selection(&self) -> &Selection {
        &self.data_selection
    }

    /// update the selection by creating a modified dataset that only contains the selected data.
    pub fn set_selection(&mut self, selected: Vec<bool>) -> Result<(), Error> {
        let orig_len = self.data_selection.len();
        let select_len = selected.len();
        if orig_len != select_len {
            Err(Error::selection_failure(select_len, orig_len))
        } else {
            self.current = if selected.iter().all(|x| *x) {
                // move back to the original which is a full selection of all data
                self.original.clone()
            } else {
                get_derived_stitched(&self.original, &selected)
            };

            // update the selection
            selected
                .into_iter()
                .enumerate()
                .for_each(|(idx, sel)| self.data_selection[idx].selected = sel);
            Ok(())
        }
    }
}
