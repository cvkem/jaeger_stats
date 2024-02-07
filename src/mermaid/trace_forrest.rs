use super::{trace_node::TraceNode, TraceData};
use crate::stats::call_chain::Call;

/// Represents the status of a Path in relation to a TraceForrest
#[derive(PartialEq)]
pub enum EmbeddingKind {
    /// Path has no embedding, so it escapes TraceForrest early
    None,
    /// Path is completely embedded in the TraceForrest
    Embedded,
    /// The Path is embedded in the TraceForrest, and extends it at a leaf-position
    Extended,
}

#[derive(Debug)]
/// TraceForrest is used to contain a series of (partial) overlapping paths and extends them to a tree-structured, such that joined segments (prefixes) as shown a a single path.
pub struct TraceForrest<'a>(pub Vec<TraceNode<'a>>);

impl<'a> TraceForrest<'a> {
    /// build a tree that based on a series of independent but overlapping paths.
    /// thus getting the shared structure visible.
    pub fn build_trace_forrest(paths: Vec<&'a TraceData>) -> Self {
        let mut trace_forrest = TraceForrest(Vec::new());
        paths.iter().for_each(|td| {
            trace_forrest.add_path(&td.trace_path.call_chain[..]);
        });
        trace_forrest
    }

    /// find the index of a tracenode that corresponds to Call at this level in the traceTree and return the index (or None)
    fn find_trace_node_idx(&self, call: &Call) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, tn)| tn.service == call.service && tn.operation == call.operation)
            .next()
            .map(|(idx, _tn)| idx)
    }

    /// aadd a new node to the Forrest based on Call
    fn add_node<'b>(&mut self, call: &'b Call) -> usize
    where
        'b: 'a,
    {
        let last_idx = self.0.len();
        self.0.push(TraceNode::new(call));
        last_idx
    }

    /// add a full path to this TraceTree (via recursion)
    fn add_path(&mut self, path: &'a [Call]) {
        // set up the loop
        let mut trace_forrest = self;
        let mut path = path;

        while !path.is_empty() {
            let head = &path[0];

            // find node, or insert new trace-node if not present.
            let idx = match trace_forrest.find_trace_node_idx(head) {
                Some(idx) => idx,
                None => trace_forrest.add_node(head),
            };

            // move to next level
            trace_forrest = &mut trace_forrest.0[idx].callees;
            path = &path[1..];
        }
    }

    /// checks if the path is an embedded path, so it is full included within this trace_forrest (but might extend beyong the leaf node of the TraceForrest)
    pub fn embedding(&self, path: &'a [Call]) -> EmbeddingKind {
        // set up the loop
        let mut trace_forrest = self;
        let mut path = path;

        loop {
            if trace_forrest.0.is_empty() {
                return EmbeddingKind::Extended;
            }
            if path.is_empty() {
                return EmbeddingKind::Embedded;
            }
            let head = &path[0];

            // find node in the curret level of the TraceForrest.
            let Some(idx) = trace_forrest.find_trace_node_idx(head) else {
                return EmbeddingKind::None; // not present so no embedding.
            };

            // move to next level
            trace_forrest = &trace_forrest.0[idx].callees;
            path = &path[1..];
        }
    }
}
