use super::{
    types::{ChartDataParameters, ProcessList, Selection, Table},
    MermaidScope,
    TraceScope,
    ViewError,
};





pub trait Viewer {

    /// Read the file (either Traces or Stitched) and create a viewer for it.
fn from_file(file_name: &str) -> Result<Box<Self>, ViewError>;

    /// Get a copy of the cached label-list
//    fn get_label_list(&self) -> Vec<String>;

    /// Get the list of processes that exist in the current dataset.
    fn get_process_list(&self, metric: &str) -> ProcessList;

    /// Get the list of call-chains for a given Service-Operation
    fn get_call_chain_list(
        &self,
        proc_oper: &str,
        metric: &str,
        scope: TraceScope,
        inbound_idx: Option<i64>,
    ) -> ProcessList;

    /// Get the chart-data for a specific service-operation 
    fn get_service_oper_chart_data(
        &self,
        process: &str,
        metric: &str,
    ) -> Option<ChartDataParameters>;

    /// get a mermaid diagram that depicts the current selection based on proc_oper and optionally a call-chain.
    fn get_mermaid_diagram(
        &self,
        service_oper: &str,
        call_chain_key: Option<&str>,
        scope: MermaidScope,
        compact: bool,
    ) -> String;

    fn get_call_chain_chart_data(
        &self,
        call_chain_key: &str,
        metric: &str,
    ) -> Option<ChartDataParameters>;

    /// filestats are always derived from the original dataset
    fn get_file_stats(&self) -> Table;

    /// get the current selection of the data-range
    fn get_selection(&self) -> &Selection;

    /// update the selection by creating a modified dataset that only contains the selected data.
    fn set_selection(&mut self, selected: Vec<bool>) -> Result<(), ViewError>;
}