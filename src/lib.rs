//#![allow(non_snake_case)]

mod processed;
mod raw;
mod stats;
mod trace_analysis;
mod utils;

mod graph;
mod stitch;

pub use graph::build_graph;
pub use raw::{
    read_file_or_folder, read_jaeger_trace_file, write_traces, JaegerItem, JaegerLog, JaegerSpan,
    JaegerTags, JaegerTrace,
};
pub use stats::{chained_stats, file::StatsRecJson, CChainEndPointCache, StatsRec};
pub use utils::{
    datetime_micros_str, datetime_millis_str, datetime_to_micros, hash, micros_to_datetime, report,
    set_comma_float, set_tz_offset_minutes, string_hash, write_report,
};

pub use stitch::{
    get_call_chain_chart_data, get_call_chain_list, get_file_stats, get_label_list,
    get_proc_oper_chart_data, get_process_list, AnomalyParameters, BestFit, ChartDataParameters,
    ChartLine, ProcessListItem, StitchList, StitchParameters, Stitched, StitchedLine, StitchedSet, Table
};
pub use trace_analysis::analyze_file_or_folder;
