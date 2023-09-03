//! Routines to read Jaeger-tracing JSON files directly via Serde

mod jaeger;
mod read_folder;
mod read_jaeger;
mod write;

pub use self::{
    jaeger::{JaegerItem, JaegerLog, JaegerSpan, JaegerTags, JaegerTrace},
    read_folder::{read_file_or_folder, read_process_file_or_folder},
    read_jaeger::read_jaeger_trace_file,
    write::write_traces,
};
