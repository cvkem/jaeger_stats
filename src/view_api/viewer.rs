use super::{
    types::{ChartDataParameters, ProcessList, Selection, Table},
    MermaidScope, Metric, TraceScope, ViewError,
};

pub trait Viewer {
    /// Read the file (either Traces or Stitched) and create a viewer for it.
    fn from_file(file_name: &str) -> Result<Box<Self>, ViewError>
    where
        Self: Sized;

    /// Does the viewer contains a time-series, and thus does it have time-series charts available?
    fn is_time_series(&self) -> bool {
        false
    }

    /// Get the list of processes that exist in the current dataset.
    fn get_process_list(&self, metric: Metric) -> ProcessList;

    /// Get the list of call-chains for a given Service-Operation
    fn get_call_chain_list(
        &self,
        proc_oper: &str,
        metric: Metric,
        scope: TraceScope,
        inbound_idx: Option<i64>,
    ) -> ProcessList;

    #[allow(unused_variables)]
    /// Get the chart-data for a specific service-operation
    fn get_service_oper_chart_data(
        &self,
        full_service_oper_key: &str,
        metric: Metric,
    ) -> Option<ChartDataParameters> {
        // default implementation does not have charts as it does not contain a time-series.
        None
    }

    #[allow(unused_variables)]
    /// Get the chart-data for a specific call-chain (exact path to a service/operation)
    fn get_call_chain_chart_data(
        &self,
        call_chain_key: &str,
        metric: Metric,
    ) -> Option<ChartDataParameters> {
        // default implementation does not have charts as it does not contain a time-series.
        None
    }

    /// filestats are always derived from the original dataset
    fn get_file_stats(&self) -> Table {
        panic!("Get_file_stats only exists for time-series data")
    }

    /// get the current selection of the data-range
    fn get_selection(&self) -> &Selection {
        panic!("Get_selection only exists for time-series data")
    }

    /// update the selection by creating a modified dataset that only contains the selected data.
    fn set_selection(&mut self, _selected: Vec<bool>) -> Result<(), ViewError> {
        panic!("Set_selection only exists for time-series data")
    }

    /// get a mermaid diagram that depicts the current selection based on proc_oper and optionally a call-chain.
    fn get_mermaid_diagram(
        &self,
        service_oper: &str,
        call_chain_key: Option<&str>,
        edge_value: Metric,
        scope: MermaidScope,
        compact: bool,
    ) -> String;
}
