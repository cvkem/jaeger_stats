//! Routines to read Jaeger-tracing JSON files directly via Serde

mod jaeger;
mod read_jaeger;

pub use self::{
    jaeger::{JaegerItem, JaegerSpan, JaegerTags, JaegerTrace},
    read_jaeger::read_jaeger_trace_file
};
