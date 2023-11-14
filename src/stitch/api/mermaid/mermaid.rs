use super::service_oper_graph::ServiceType;



pub enum MermaidNode {
    Node(String),
    SubGraph(String, ServiceType, Vec<MermaidNode>),
}