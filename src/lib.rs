//#![allow(non_snake_case)]

mod mermaid;
mod processed;
mod raw;
mod stats;
mod trace_analysis;
pub mod utils;

mod graph;
mod stitch;

pub use graph::build_graph;
pub use mermaid::MermaidScope;
pub use raw::{
    read_file_or_folder, read_jaeger_trace_file, write_traces, JaegerItem, JaegerLog, JaegerSpan,
    JaegerTags, JaegerTrace,
};
pub use stats::{chained_stats, file::StatsRecJson, CChainEndPointCache, StatsRec};
pub use trace_analysis::EdgeValue;
pub use utils::{
    current_folder, datetime_micros_str, datetime_millis_str, datetime_to_micros, hash,
    micros_to_datetime, report, set_comma_float, set_tz_offset_minutes, string_hash, write_report,
};

pub use stitch::{
    AnomalyParameters, BestFit, ChartDataParameters, ChartLine, ProcessListItem, Selection,
    StitchList, StitchParameters, Stitched, StitchedDataSet, StitchedLine, StitchedSet, Table,
};
pub use trace_analysis::analyze_file_or_folder;
