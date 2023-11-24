use super::{
    super::flowchart::{Mermaid, MermaidBasicNode, MermaidLink, MermaidSubGraph},
    compact_link::{CompKey, CompValue, CompactLink},
    operation::Operation,
    position::Position,
    service_oper_graph::ServiceOperGraph,
    service_oper_type::ServiceOperationType,
};
use crate::stats::call_chain::{Call, CallDirection};

#[derive(Debug)]
pub struct Service {
    pub service: String,
    pub serv_oper_type: ServiceOperationType,
    pub operations: Vec<Operation>,
    pub position: Position,
}

impl Service {
    /// Create a new service with the given name on the provided Position.
    /// If Position is Centered the Service will be labelled as 'ServiceOperationType::Emphasized'
    pub fn new(service: String, position: Position) -> Self {
        let serv_oper_type = match position {
            Position::Center => ServiceOperationType::Emphasized,
            _ => ServiceOperationType::Default,
        };
        Self {
            service,
            serv_oper_type,
            operations: Vec::new(),
            position,
        }
    }

    /// push an Operation to the Servic and return the index
    pub fn add_operation(&mut self, oper: String, call_direction: CallDirection) -> usize {
        let oper_idx = self.operations.len();
        self.operations.push(Operation::new(oper, call_direction));
        oper_idx
    }

    /// add this service as a subgraph with a series of nodes
    pub fn mermaid_add_service_oper(&self, mermaid: &mut Mermaid) {
        let mut sub_graph = MermaidSubGraph::new(self.service.clone(), self.serv_oper_type);

        self.operations.iter().for_each(|oper| {
            sub_graph.add_node(MermaidBasicNode::new(
                format!("{}/{}", &self.service, &oper.oper),
                oper.serv_oper_type,
            ))
        });
        mermaid.add_subgraph(sub_graph);
    }

    /// add this process as a subgraph with a series of nodes
    pub fn mermaid_add_service_oper_links(&self, mermaid: &mut Mermaid, sog: &ServiceOperGraph) {
        self.operations.iter().for_each(|oper| {
            oper.calls.iter().for_each(|call| {
                let src = format!("{}/{}", self.service, oper.oper);
                let target = sog.get_target(call.to_service, call.to_oper);
                mermaid.add_link(MermaidLink::new(src, target, call.count, call.line_type));
            })
        });
    }

    /// add this service as a node
    pub fn mermaid_add_service(&self, mermaid: &mut Mermaid) {
        let service_node = MermaidBasicNode::new(self.service.clone(), self.serv_oper_type);
        mermaid.add_node(service_node);
    }

    /// add this process as a subgraph with a series of nodes
    pub fn mermaid_add_service_links(&self, mermaid: &mut Mermaid, sog: &ServiceOperGraph) {
        let mut compact_link = CompactLink::new();

        self.operations.iter().for_each(|oper| {
            oper.calls.iter().for_each(|call| {
                let src = format!("{}/{}", self.service, oper.oper);
                let target = sog.get_service(call.to_service);
                compact_link.add(
                    CompKey::new(target),
                    CompValue::new(call.count, call.line_type),
                )
            })
        });

        compact_link.0.into_iter().for_each(|(k, v)| {
            mermaid.add_link(MermaidLink::new(
                self.service.clone(),
                k.target.to_string(),
                v.count,
                v.link_type,
            ))
        })
    }

    /// Get the label of an operation (or outbound call) of this process
    pub fn get_operation_label(&self, oper_idx: usize) -> String {
        format!("{}/{}", self.service, self.operations[oper_idx].oper)
    }
}
