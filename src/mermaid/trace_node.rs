use super::trace_forrest::TraceForrest;
use crate::stats::call_chain::Call;

#[derive(Debug)]
pub struct TraceNode<'a> {
    pub service: &'a str,
    pub operation: &'a str,
    pub callees: TraceForrest<'a>,
}

impl<'a> TraceNode<'a> {
    /// create a new TraceNode that refers the service/operation of the provided Call.
    pub fn new(call: &'a Call) -> Self {
        Self {
            service: &call.service,
            operation: &call.operation,
            callees: TraceForrest(Vec::new()),
        }
    }
}
