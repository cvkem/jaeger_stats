use super::{
    super::Stitched,
    selection::{get_derived_stitched, get_full_selection},
    utils,
};
use crate::{
    mermaid,
    view_api::types::{ChartDataParameters, ProcessList, Selection, Table},
    MermaidScope, Metric, TraceScope, ViewError, Viewer,
};
use log::{error, info};
use std::{path::Path, sync::Arc};

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

    /// Get a copy of the cached label-list
    fn get_label_list(&self) -> Vec<String> {
        self.data_selection
            .iter()
            .filter(|label_item| label_item.selected)
            .map(|label_item| label_item.label.to_owned())
            .collect()
    }
}

impl Viewer for StitchedDataSet {
    fn from_file(file_name: &str) -> Result<Box<Self>, ViewError> {
        if Path::new(file_name).exists() {
            info!("Trying to load the file {file_name}");

            match Stitched::from_json(file_name) {
                Ok(data) => {
                    info!("Ready loading file");
                    Ok(Box::new(Self::new(data)))
                }
                Err(err) => Err(ViewError::load_failure(
                    file_name.to_owned(),
                    format!("{err:?}"),
                )),
            }
        } else {
            let msg = format!("ERROR: File '{file_name} does not exist");
            error!("{msg}");
            Err(ViewError::does_not_exist(file_name.to_owned()))
        }
    }

    /// Does the viewer contains a time-series, and thus does it have time-series charts available?
    fn time_series(&self) -> bool {
        true
    }

    fn get_process_list(&self, metric: Metric) -> ProcessList {
        utils::get_process_list(&self.current, metric)
    }

    fn get_call_chain_list(
        &self,
        proc_oper: &str,
        metric: Metric,
        scope: TraceScope,
        inbound_idx: Option<i64>,
    ) -> ProcessList {
        utils::get_call_chain_list(&self.current, proc_oper, metric, scope, inbound_idx)
    }

    /// Get the chart-data for a specific service-operation
    fn get_service_oper_chart_data(
        &self,
        full_service_oper_key: &str,
        metric: Metric,
    ) -> Option<ChartDataParameters> {
        utils::get_service_oper_chart_data(
            &self.current,
            self.get_label_list(),
            full_service_oper_key,
            metric,
        )
    }

    /// get a mermaid diagram that depicts the current selection based on proc_oper and optionally a call-chain.
    fn get_mermaid_diagram(
        &self,
        service_oper: &str,
        call_chain_key: Option<&str>,
        edge_value: Metric,
        scope: MermaidScope,
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
                            .find(|x| x.metric == Metric::AvgDurationMillis)
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
            service_oper,
            call_chain_key,
            edge_value,
            scope,
            compact,
        )
    }

    fn get_call_chain_chart_data(
        &self,
        call_chain_key: &str,
        metric: Metric,
    ) -> Option<ChartDataParameters> {
        utils::get_call_chain_chart_data(
            &self.current,
            self.get_label_list(),
            call_chain_key,
            metric,
        )
    }

    /// filestats are always derived from the original dataset
    fn get_file_stats(&self) -> Table {
        utils::get_file_stats(&self.original)
    }

    fn get_selection(&self) -> &Selection {
        &self.data_selection
    }

    /// update the selection by creating a modified dataset that only contains the selected data.
    fn set_selection(&mut self, selected: Vec<bool>) -> Result<(), ViewError> {
        let orig_len = self.data_selection.len();
        let select_len = selected.len();
        if orig_len != select_len {
            Err(ViewError::selection_failure(select_len, orig_len))
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
