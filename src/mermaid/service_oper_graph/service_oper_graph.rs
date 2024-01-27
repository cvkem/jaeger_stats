use super::{
    super::flowchart::Mermaid,
    super::MermaidScope,
    call_descriptor::CallDescriptor,
    edge_value_selector::{edge_value_to_selector, EdgeValueSelector},
    link_type::LinkType,
    loc::Loc,
    node_select::{scope_to_node_selector, NodeSelector},
    operation::Operation,
    position::Position,
    service::Service,
    service_oper_type::ServiceOperationType,
};
use crate::{mermaid::trace_data::TraceDataStats, stats::call_chain::Call, EdgeValue};

/// A ServiceOperGraph is a vector of service that each contain a vector of Operations. Each Operation collects data on the set of outbound calls
#[derive(Debug)]
pub struct ServiceOperGraph(Vec<Service>);

impl ServiceOperGraph {
    pub fn new() -> Self {
        ServiceOperGraph(Vec::new())
    }

    /// insert a new Service-operation pair and return its location within the ServiceOperGraph
    fn add_service_operation(&mut self, call: Call, position: Position) -> Loc {
        let mut service = Service::new(call.process, position);
        let serv_idx = self.0.len();
        let oper_idx = service.add_operation(call.method, call.call_direction);
        self.0.push(service);
        Loc {
            service_idx: serv_idx,
            oper_idx,
        }
    }

    /// Get the index of a service
    fn get_service_idx(&self, service_name: &str) -> Option<usize> {
        self.0.iter().position(|p| &p.service[..] == service_name)
    }

    /// find the Service-Operation combination, and return the index-pair as a Location in the ServiceOperGraph or None
    fn get_service_operation_idx(&self, call: &Call) -> Option<Loc> {
        match self.get_service_idx(&call.process) {
            Some(serv_idx) => match self.0[serv_idx]
                .operations
                .iter()
                .position(|o| o.oper == call.method)
            {
                Some(oper_idx) => Some(Loc {
                    service_idx: serv_idx,
                    oper_idx,
                }),
                None => None,
            },
            None => None,
        }
    }

    /// find the Service-Operation combination, or insert it, and return the index-pair as a Location in the ServiceOperGraph
    fn get_create_service_operation_idx(&mut self, call: Call, position: Position) -> Loc {
        if let Some(serv_idx) = self.get_service_idx(&call.process) {
            let service = &mut self.0[serv_idx];
            service.position = service.position.check_relevance(position);
            match service
                .operations
                .iter()
                .position(|o| o.oper == call.method)
            {
                Some(oper_idx) => Loc {
                    service_idx: serv_idx,
                    oper_idx,
                },
                None => {
                    let oper_idx = self.0[serv_idx].add_operation(call.method, call.call_direction);
                    Loc {
                        service_idx: serv_idx,
                        oper_idx,
                    }
                }
            }
        } else {
            self.add_service_operation(call, position)
        }
    }

    /// Add a connection between 'from' and 'to' where the incoming edge is labeled with edge_value.
    /// In case of calls between services this is expected to be an outbound call from the sender (from) and an inbound call for the receiver.
    /// However, when from and to are located in the same service it is a connection is from the receiver (inbound) to the sender (outbound) as it is an internal pass-through.
    pub fn add_connection(
        &mut self,
        from: Call,
        to: Call,
        data: &TraceDataStats,
        service: &str,
        default_pos: Position,
    ) {
        // determine the from and to and add them if they do not exist
        let (from_pos, to_pos) = Position::find_positions(&from, &to, service, default_pos);
        let from = self.get_create_service_operation_idx(from, from_pos);
        let to = self.get_create_service_operation_idx(to, to_pos);
        // Add new link or update the existing link with the data
        self.0[from.service_idx].operations[from.oper_idx].upsert_link(to, data)
    }

    /// update the LineType of the given connection. The connection should exist, otherwise it is created.
    pub fn update_line_type(&mut self, from: &Call, to: &Call, line_type: LinkType) {
        // determine the from and to only if they exist
        let from_loc = self.get_service_operation_idx(from);
        let to_loc = self.get_service_operation_idx(to);
        // Update the link-type
        match (from_loc, to_loc) {
            (Some(from), Some(to)) => {
                self.0[from.service_idx].operations[from.oper_idx].update_line_type(to, line_type)
            }
            (None, None) => println!("Failed to find both {from:?} and {to:?}"),
            (None, Some(_)) => println!("Failed to find from:{from:?} in update_line_type"),
            (Some(_), None) => println!("Failed to find to:{to:?} in update_line_type"),
        }
    }

    // CODE IS REFACTORED BELOW to MAKE it more linear
    // /// Update the serv_oper_type of a service_operation
    // pub fn update_service_operation_type(
    //     &mut self,
    //     service_name: &str,
    //     serv_oper_type: ServiceOperationType,
    // ) {
    //     match self.get_service_operation_idx(
    //         Call::extract_call(service_name)
    //             .as_ref()
    //             .expect("could not find service/operation"),
    //     ) {
    //         Some(loc) => self.0[loc.service_idx].operations[loc.oper_idx]
    //             .update_serv_oper_type(serv_oper_type),
    //         None => panic!("Could not find service '{service_name}' index to update serv_oper_type"),
    //     }
    // }

    /// Update the serv_oper_type of a service_operation
    pub fn update_service_operation_type(
        &mut self,
        service_name: &str,
        serv_oper_type: ServiceOperationType,
    ) {
        Call::extract_call(service_name)
            .as_ref()
            .and_then(|call| self.get_service_operation_idx(call))
            .map(|loc| self.0[loc.service_idx].operations[loc.oper_idx]
                .update_serv_oper_type(serv_oper_type))
            .unwrap_or_else(|| panic!("Could not find service '{service_name}' to update serv_oper_type to '{serv_oper_type:?}'"))
    }

    /// Update the inbound_path_count for the call from-to
    pub fn update_inbound_path_count(
        &mut self,
        from: &Call,
        to: &Call,
        edge_data: &TraceDataStats,
    ) {
        // determine the from and to only if they exist
        let from_loc = self.get_service_operation_idx(from);
        let to_loc = self.get_service_operation_idx(to);
        // Update the link-type
        match (from_loc, to_loc) {
            (Some(from), Some(to)) => self.0[from.service_idx].operations[from.oper_idx]
                .update_inbound_path_count(to, edge_data),
            (None, None) => println!("Failed to find both {from:?} and {to:?}"),
            (None, Some(_)) => {
                println!("Failed to find from:{from:?} in update_inbound_path_count")
            }
            (Some(_), None) => println!("Failed to find to:{to:?} in update_inbound_path_count"),
        }
    }

    /// get the name of a target defined by serv_idx and oper_idx within this Graph.
    pub fn get_target(&self, serv_idx: usize, oper_idx: usize) -> String {
        self.0[serv_idx].get_operation_label(oper_idx)
    }

    /// get a shared reference to the operation defined by serv_idx and oper_idx within this Graph.
    pub fn get_Operation(&self, serv_idx: usize, oper_idx: usize) -> &Operation {
        self.0[serv_idx].get_operation(oper_idx)
    }

    /// get the 'Service' for this idx
    pub fn get_service(&self, serv_idx: usize) -> &Service {
        &self.0[serv_idx]
    }

    /// generate a detailled Mermaid diagram, which includes the operations and the outbound calls of each of the services.
    fn mermaid_diagram_full(
        &self,
        node_select: NodeSelector,
        edge_value_selector: EdgeValueSelector,
    ) -> String {
        let mut mermaid = Mermaid::new();

        self.0
            .iter()
            .for_each(|p| p.mermaid_add_service_oper(&mut mermaid));
        self.0.iter().for_each(|p| {
            p.mermaid_add_service_oper_links(&mut mermaid, self, node_select, edge_value_selector)
        });

        mermaid.to_diagram()
    }

    /// Get a compact Mermaid diagram only showing the services, and discarding the detail regarding the actual operation being called.
    fn mermaid_diagram_compact(
        &self,
        node_select: NodeSelector,
        edge_value_selector: EdgeValueSelector,
    ) -> String {
        let mut mermaid = Mermaid::new();

        self.0
            .iter()
            .filter(|serv| node_select(serv))
            .for_each(|p| p.mermaid_add_service(&mut mermaid));
        self.0
            .iter()
            .filter(|serv| node_select(serv))
            .for_each(|p| {
                p.mermaid_add_service_links(&mut mermaid, self, node_select, edge_value_selector)
            });

        mermaid.to_diagram()
    }

    /// Extract the mermaid diagram based on these imputs
    pub fn mermaid_diagram(
        &self,
        scope: MermaidScope,
        compact: bool,
        edge_value: EdgeValue,
    ) -> String {
        let node_select = scope_to_node_selector(scope);
        let edge_value_selector = edge_value_to_selector(edge_value);
        if compact {
            self.mermaid_diagram_compact(node_select, edge_value_selector)
        } else {
            self.mermaid_diagram_full(node_select, edge_value_selector)
        }
    }
}
