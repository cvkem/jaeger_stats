//! Generate a clean and processed Trace-object (including Spans) out of a raw Jaeger trace.
mod trace;
mod span;
mod process_map;

pub use self::{
    trace::{extract_traces, Trace},
    span::Spans
};