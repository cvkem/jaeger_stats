use crate::{
    mermaid,
    stats::StatsRec,
    utils::{TimeStats, write_string_to_file},
    view_api::types::ProcessList,
    EdgeValue, MermaidScope, TraceScope, ViewError, Viewer,
};
use super::utils;
use log::{error, info};
use std::path::Path;

pub struct TraceDataSet(pub StatsRec);

impl TraceDataSet {
    pub fn new(data: StatsRec) -> Self {
        Self(data)
    }

    /// get a diagram and write it to a folder
    pub fn write_mermaid_diagram(
        &self,
        folder: &Path,
        service_oper: &str,
        call_chain_key: Option<&str>,
        edge_value: EdgeValue,
        scope: MermaidScope,
        compact: bool,
    ) {
        let diagram =
            self.get_mermaid_diagram(service_oper, call_chain_key, edge_value, scope, compact);
        write_diagram(folder, service_oper, diagram);
    }
}

impl Viewer for TraceDataSet {
    fn from_file(file_name: &str) -> Result<Box<Self>, ViewError> {
        if Path::new(file_name).exists() {
            info!("Trying to load the file {file_name}");

            let file_path = Path::new(file_name).to_path_buf().into_os_string(); //get_full_path(base_path, input);
            match StatsRec::read_file(&file_path) {
                Ok(stats_rec) => Ok(Box::new(TraceDataSet::new(stats_rec))),
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

    /// Get the list of processes that exist in the current dataset.
    fn get_process_list(&self, metric: &str) -> ProcessList {
        utils::get_process_list(&self.0, metric)
    }

    /// Get the list of call-chains for a given Service-Operation
    fn get_call_chain_list(
        &self,
        proc_oper: &str,
        metric: &str,
        scope: TraceScope,
        inbound_idx: Option<i64>,
    ) -> ProcessList {
        utils::get_call_chain_list(&self.0, proc_oper, metric, scope, inbound_idx)
    }

    fn get_mermaid_diagram(
        &self,
        service_oper: &str,
        call_chain_key: Option<&str>,
        edge_value: EdgeValue,
        scope: MermaidScope,
        compact: bool,
    ) -> String {
        let trace_tree = self
            .0
            .stats
            .iter()
            .map(|(service_oper, oper_stats)| {
                let trace_data = oper_stats
                    .call_chain
                    .0
                    .iter()
                    .map(|(cck, ccv)| {
                        let key = cck.call_chain_key();
                        let count = ccv.count as u64;
                        let avg_duration_millis = TimeStats(&ccv.duration_micros).get_avg_millis();
                        let p75_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.75);
                        let p90_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.90);
                        let p95_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.95);
                        let p99_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.99);

                        // TODO: Made switch to aggregator at the wrong site. this is still a tree. Move it to get_diagram
                        mermaid::TraceData::new(
                            &key,
                            ccv.rooted,
                            cck.is_leaf,
                            count,
                            avg_duration_millis,
                            p75_millis,
                            p90_millis,
                            p95_millis,
                            p99_millis,
                        )
                    })
                    .collect();
                (service_oper.clone(), trace_data)
            })
            .collect();

        let diagram = mermaid::TracePaths(trace_tree).get_diagram(
            service_oper,
            call_chain_key,
            edge_value,
            scope,
            compact,
        );
        diagram
    }
}

/// helper function used internally
fn write_diagram(folder: &Path, proc_oper: &str, diagram: String) {
    let mut file_path = folder.to_path_buf();

    let clean_proc_oper = proc_oper.replace(['/', '\\'], "_"); // The / is not allowed in a file-path
    file_path.push(format!("{}.mermaid", clean_proc_oper));
    let file_path = file_path.to_str().expect("invalid file-path");
    if let Err(err) = write_string_to_file(file_path, diagram) {
        panic!("Writing to file '{file_path}' failed with error: {err:?}");
    };
}
