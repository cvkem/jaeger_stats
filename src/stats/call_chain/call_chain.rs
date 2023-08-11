#![allow(clippy::module_inception)]
use super::call::Call;
use crate::processed::{Span, Spans, SpansExt};

pub type CallChain = Vec<Call>;

/// get_call_chain returns the full call_chain from top to bottom showing process and called method
/// this function does a recursive trace back to identify all parent-links:
pub fn get_call_chain(idx: usize, spans: &Spans) -> CallChain {
    fn processor(span: &Span) -> Call {
        let process = span.get_process_str().to_owned();
        let method = span.operation_name.to_owned();
        let call_direction = span.span_kind.as_ref().into();
        Call {
            process,
            method,
            call_direction,
        }
    }
    SpansExt(spans).chain_apply_forward(idx, &processor)
}
