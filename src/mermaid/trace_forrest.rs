use super::{trace_node::TraceNode, TraceData};
use crate::stats::call_chain::Call;

// pub struct TraceNode<'a> {
//     pub service: &'a str,
//     pub oper: &'a str,
//     pub callees: TraceTree<'a>,
// }

#[derive(Debug)]
/// TraceForrest is used to contain a series of (partial) overlapping paths and extends them to a tree-structured, such that joined segments (prefixes) as shown a a single path.
pub struct TraceForrest<'a>(pub Vec<TraceNode<'a>>);

impl<'a> TraceForrest<'a> {
    /// build a tree that based on a series of independent but overlapping paths.
    /// thus getting the shared structure visible.
    pub fn build_trace_tree(paths: Vec<&'a TraceData>) -> Self {
        let mut trace_tree = TraceForrest(Vec::new());
        paths.iter().for_each(|td| {
            trace_tree.add_path(&td.trace_path.call_chain[..]);
        });
        trace_tree
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
    where 'b: 'a {
        let last_idx = self.0.len();
        self.0.push(TraceNode::new(call));
        last_idx
    }

    /// add a full path to this TraceTree (via recursion)
    fn add_path(&mut self, path: &'a [Call]) {
        // set up the loop
        let mut trace_tree = self;
        let mut path = path;

        while !path.is_empty() {
            let head = &path[0];

            // find node, or inssert new trace-node if not present.
            let idx = match trace_tree.find_trace_node_idx(head) {
                Some(idx) => idx,
                None => trace_tree.add_node(head)
            };

            // move to next level
            trace_tree = &mut trace_tree.0[idx].callees;
            path = &path[1..];
        }
    }
}
