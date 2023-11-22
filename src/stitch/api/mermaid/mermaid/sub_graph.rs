use super::{
    basic_node::MermaidBasicNode,
    indent::INDENT_STR,
    super::service_oper_graph::{LinkType, ServiceOperationType},
    node::MermaidNode,
};

/// A node with an inner structure representing nested nodes.
pub struct MermaidSubGraph<'a> {
    /// service has a plain name, so no embedded '/' is expected.
    service: &'a str,
    serv_oper_type: ServiceOperationType,
    nodes: Vec<MermaidNode<'a>>,
}

impl<'a> MermaidSubGraph<'a> {
    /// add a subgraph, a container that can contain nested simple nodes and subgraphs
    pub fn add_subgraph(&mut self, sg: MermaidSubGraph<'a>) {
        self.nodes.push(MermaidNode::SubGraph(sg))
    }

    /// add a simples node (without any nested nodes)
    pub fn add_node(&mut self, node: MermaidBasicNode<'a>) {
        self.nodes.push(MermaidNode::Node(node))
    }

    /// to_diagram
    /// TODO handle indentation
    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        diagram.push(format!(
            "{}subgraph {}",
            INDENT_STR.get_indent_str(indent),
            self.service
        ));
        self.nodes
            .iter()
            .for_each(|node| node.to_diagram(diagram, indent + 1));
        diagram.push(format!("{}end", INDENT_STR.get_indent_str(indent)));

        if self.serv_oper_type == ServiceOperationType::Emphasized {
            diagram.push(format!("\tstyle {} fill:#00f", self.service))
        };
    }
}
