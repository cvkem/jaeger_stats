use super::service_oper_graph::{Service, ServiceOperationType};

pub enum MermaidNode {
    Node(String, ServiceOperationType),
    SubGraph(String, ServiceOperationType, Vec<MermaidNode>),
}

struct MermaidLink {}
