use super::service_oper_graph::{LinkType, Service, ServiceOperationType};

pub struct MermaidService {
    service: String,
    serv_oper_type: ServiceOperationType,
}

pub struct MermaidSubGraph {
    service: String,
    serv_oper_type: ServiceOperationType,
    nodes: Vec<MermaidNode>,
}

pub enum MermaidNode {
    Node(MermaidService),
    SubGraph(MermaidSubGraph),
}

struct MermaidLink {
    src: String,
    target: String,
    value: f64,
    link_type: LinkType,
}

struct Mermaid {
    nodes: Vec<MermaidNode>,
    links: Vec<MermaidLink>,
}
