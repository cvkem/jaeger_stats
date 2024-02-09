use super::{
    super::service_oper_graph::ServiceOperationType, basic_node::MermaidBasicNode,
    escape_name::escape_mermaid_label, indent::INDENT_STR, node::MermaidNode,
};

/// A node with an inner structure representing nested nodes.
pub struct MermaidSubGraph {
    /// service has a plain name, so no embedded '/' is expected.
    service: String,
    serv_oper_type: ServiceOperationType,
    nodes: Vec<MermaidNode>,
}

impl MermaidSubGraph {
    pub fn new(service: String, serv_oper_type: ServiceOperationType) -> Self {
        Self {
            service,
            serv_oper_type,
            nodes: Vec::new(),
        }
    }

    // /// add a subgraph, a container that can contain nested simple nodes and subgraphs
    // pub fn add_subgraph(&mut self, sg: MermaidSubGraph) {
    //     self.nodes.push(MermaidNode::SubGraph(sg))
    // }

    /// add a simples node (without any nested nodes)
    pub fn add_node(&mut self, node: MermaidBasicNode) {
        self.nodes.push(MermaidNode::Node(node))
    }

    /// to_diagram
    /// TODO handle indentation
    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        let esc_service = escape_mermaid_label(&self.service);
        let indent_str = INDENT_STR.get_indent_str(indent);
        let subgraph_spec = match esc_service.as_ref() {
            None => format!("{}subgraph {}", indent_str, self.service),
            Some(esc_service) => format!(
                "{}subgraph {} [\"{}\"]",
                indent_str, esc_service, self.service
            ),
        };
        diagram.push(subgraph_spec);
        self.nodes
            .iter()
            .for_each(|node| node.to_diagram(diagram, indent + 1));
        diagram.push(format!("{}end", INDENT_STR.get_indent_str(indent)));

        if self.serv_oper_type == ServiceOperationType::Emphasized {
            let the_service = match esc_service.as_ref() {
                Some(esc_service) => esc_service,
                None => &self.service,
            };
            diagram.push(format!("\tstyle {} fill:#00b33c", the_service))
        };
    }
}
