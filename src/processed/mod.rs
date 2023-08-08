//! Generate a clean and processed Trace-object (including Spans) out of a raw Jaeger trace.
mod process_map;
mod span;
mod trace;

pub use self::{
    span::{Span, Spans},
    trace::{extract_traces, Trace},
};
