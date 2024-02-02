use super::{trace_node::TraceNode, TraceData};
use crate::stats::call_chain::Call;

// pub struct TraceNode<'a> {
//     pub service: &'a str,
//     pub oper: &'a str,
//     pub callees: TraceTree<'a>,
// }

#[derive(Debug)]
/// TraceTrace is used to contain a series of (partial) overlapping paths and extends them to a tree-structured, such that joined segments (prefixes) as shown a a single path.
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

    // /// add a call to this TraceTree at the current level
    // fn find_insert_trace_node(&mut self, call: &'a Call) -> &mut TraceNode {
    //     let trace_node = self.0
    //         .iter_mut()
    //         .filter(|tn| tn.service == call.service && tn.operation == call.operation)
    //         .next();
    //     let trace_node = match trace_node {
    //         Some(tn) => tn,
    //         None => {
    //             let tn = TraceNode::new(call);
    //             self.0.push(tn);
    //             &mut self.0[ self.0.len()]
    //         }
    //     };
    //     trace_node
    // }

    /// add a full path to this TraceTree (via recursion)
    fn add_path(&mut self, path: &'a [Call]) {
        if !path.is_empty() {
            //            let node = self.find_insert_trace_node(&path[0]);
            // code included to circumvent lifetime issues.

            let call = &path[0];
            let trace_node = self
                .0
                .iter_mut()
                .filter(|tn| tn.service == call.service && tn.operation == call.operation)
                .next();
            let trace_node = match trace_node {
                Some(tn) => tn,
                None => {
                    let tn = TraceNode::new(call);
                    self.0.push(tn);
                    let tail_pos = self.0.len() - 1;
                    &mut self.0[tail_pos]
                }
            };

            trace_node.callees.add_path(&path[1..]);
        }
    }
}
