use super::{
    indent::INDENT_STR,
    super::service_oper_graph::{LinkType, ServiceOperationType},
    basic_node::MermaidBasicNode,
    sub_graph::MermaidSubGraph,
};


/// container to allow for BasicNodes and SubGraphs to be held in the same list
pub enum MermaidNode<'a> {
    Node(MermaidBasicNode<'a>),
    SubGraph(MermaidSubGraph<'a>),
}

impl<'a> MermaidNode<'a> {
    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        match self {
            MermaidNode::Node(node) => node.to_diagram(diagram, indent),
            MermaidNode::SubGraph(sg) => sg.to_diagram(diagram, indent),
        }
    }
}
