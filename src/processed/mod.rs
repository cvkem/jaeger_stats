//! Generate a clean and processed Trace-object (including Spans) out of a raw Jaeger trace.
mod process_map;
mod span;
mod trace;
mod unify_operation;

pub use self::{
    span::{set_max_log_msg_length, Span, Spans, SpansExt},
    trace::{extract_traces, Trace},
};
