use crate::stats::call_chain::{Call, CallDirection};

#[derive(Debug)]
struct Loc {
    service_idx: usize,
    oper_idx: usize,
}

#[derive(Debug, PartialEq)]
pub enum LinkType {
    Default,
    Reachable,
    CurrentReach,
    Emphasized,
}
// link-style example: linkStyle 37,10,22 stroke:#ff3,stroke-width:4px,color:red;

#[derive(Debug, PartialEq)]
pub enum ServiceOperationType {
    Default,
    Emphasized,
}

/// defines a position in the ProcessConnection, where the first index is the process and the second is the operation.
#[derive(Debug)]
struct CallDescriptor {
    to_service: usize,
    to_oper: usize,
    count: f64,
    line_type: LinkType,
}

impl CallDescriptor {
    fn new(loc: Loc, count: f64) -> Self {
        Self {
            to_service: loc.service_idx,
            to_oper: loc.oper_idx,
            count,
            line_type: LinkType::Default,
        }
    }
}

/// Used to store outbound
#[derive(Debug)]
pub struct Operation {
    pub oper: String,
    pub call_direction: CallDirection,
    pub serv_oper_type: ServiceOperationType,
    calls: Vec<CallDescriptor>,
}

impl Operation {
    /// Insert a link to the CallDescriptor 'to', or update it if is exists by adding the count of the 'to' CallDescriptor
    fn upsert_link(&mut self, to: CallDescriptor) {
        match self
            .calls
            .iter()
            .position(|call| call.to_oper == to.to_oper && call.to_service == to.to_service)
        {
            Some(idx) => self.calls[idx].count += to.count,
            None => self.calls.push(to),
        }
    }

    /// Update the LineType of the connector
    fn update_line_type(&mut self, to: Loc, line_type: LinkType) {
        match self
            .calls
            .iter()
            .position(|call| call.to_oper == to.oper_idx && call.to_service == to.service_idx)
        {
            Some(idx) => self.calls[idx].line_type = line_type,
            None => panic!("Could not locate a connection to {to:?}"),
        }
    }

    /// update the serv_oper_type of the current operation
    fn update_serv_oper_type(&mut self, serv_oper_type: ServiceOperationType) {
        self.serv_oper_type = serv_oper_type
    }
}

#[derive(Debug)]
pub struct Service {
    pub service: String,
    pub serv_oper_type: ServiceOperationType,
    pub operations: Vec<Operation>,
}

impl Service {
    fn new(service: String) -> Self {
        Self {
            service,
            serv_oper_type: ServiceOperationType::Default,
            operations: Vec::new(),
        }
    }

    /// push an Operation to the Servic and return the index
    fn add_operation(&mut self, oper: String, call_direction: CallDirection) -> usize {
        let oper_idx = self.operations.len();
        self.operations.push(Operation {
            oper,
            call_direction,
            serv_oper_type: ServiceOperationType::Default,
            calls: Vec::new(),
        });
        oper_idx
    }

    /// add this service as a subgraph with a series of nodes
    fn mermaid_add_nodes(&self, diagram: &mut Vec<String>) {
        diagram.push(format!("\tsubgraph {}", self.service));
        self.operations.iter().for_each(|oper| {
            diagram.push(format!(
                "\t\t{}/{}([{}/{}])",
                self.service, oper.oper, self.service, oper.oper
            ))
        });
        diagram.push("\tend".to_string());

        if self.serv_oper_type == ServiceOperationType::Emphasized {
            diagram.push(format!("\tstyle {} fill:#00f", self.service))
        };
    }

    /// add this process as a subgraph with a series of nodes
    fn mermaid_add_links(&self, diagram: &mut Vec<String>, pog: &ServiceOperGraph) {
        self.operations.iter().for_each(|oper| {
            oper.calls.iter().for_each(|call| {
                let target = pog.get_target(call.to_service, call.to_oper);
                let link = match call.line_type {
                    LinkType::Emphasized => format!(
                        "\t{}/{} ==>|{}| {}",
                        self.service, oper.oper, call.count, target
                    ),
                    _ => format!(
                        "\t{}/{} -->|{}| {}",
                        self.service, oper.oper, call.count, target
                    ),
                };
                diagram.push(link)
            })
        });
    }
    /// Get the label of an operation (or outbound call) of this process
    fn get_operation_label(&self, oper_idx: usize) -> String {
        format!("{}/{}", self.service, self.operations[oper_idx].oper)
    }
}

#[derive(Debug)]
pub struct ServiceOperGraph(Vec<Service>);

impl ServiceOperGraph {
    pub fn new() -> Self {
        ServiceOperGraph(Vec::new())
    }

    /// insert a new Service-operation pair and returns its location withig the ServiceOperGraph
    fn add_service_operation(&mut self, call: Call) -> Loc {
        let mut process = Service::new(call.process);
        let proc_idx = self.0.len();
        let oper_idx = process.add_operation(call.method, call.call_direction);
        self.0.push(process);
        Loc {
            service_idx: proc_idx,
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
            Some(proc_idx) => match self.0[proc_idx]
                .operations
                .iter()
                .position(|o| o.oper == call.method)
            {
                Some(oper_idx) => Some(Loc {
                    service_idx: proc_idx,
                    oper_idx,
                }),
                None => None,
            },
            None => None,
        }
    }

    /// find the Service-Operation combination, or insert it, and return the index-pair as a Location in the ServiceOperGraph
    fn get_create_service_operation_idx(&mut self, call: Call) -> Loc {
        if let Some(proc_idx) = self.get_service_idx(&call.process) {
            match self.0[proc_idx]
                .operations
                .iter()
                .position(|o| o.oper == call.method)
            {
                Some(oper_idx) => Loc {
                    service_idx: proc_idx,
                    oper_idx,
                },
                None => {
                    let oper_idx = self.0[proc_idx].add_operation(call.method, call.call_direction);
                    Loc {
                        service_idx: proc_idx,
                        oper_idx,
                    }
                }
            }
        } else {
            self.add_service_operation(call)
        }
    }

    /// Add a connection between 'from' and 'to'.
    /// In case of calls between services this is expected to be an outbound call from the sender (from) and an inbound call for the receiver.
    /// However, when from and to are located in the same service it is a connection is from the receiver (inbound) to the sender (outbound) as it is an internal pass-through.
    pub fn add_connection(&mut self, from: Call, to: Call, count: f64) {
        // determine the from and to and add them if they do not exist
        let from = self.get_create_service_operation_idx(from);
        let to = self.get_create_service_operation_idx(to);
        // Add or update the link
        let to = CallDescriptor::new(to, count);
        self.0[from.service_idx].operations[from.oper_idx].upsert_link(to)
    }

    /// update the LineType of the given connection. The connection should exist, otherwise it is created.
    pub fn update_line_type(&mut self, from: &Call, to: &Call, line_type: LinkType) {
        // determine the from and to and add them if they do not exist
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

    /// Update the serv_oper_type of a service_operation to Emphasized
    pub fn update_service_operation_type(
        &mut self,
        service_name: &str,
        serv_oper_type: ServiceOperationType,
    ) {
        match self.get_service_operation_idx(
            Call::extract_call(service_name)
                .as_ref()
                .expect("could not find service/operation"),
        ) {
            Some(loc) => self.0[loc.service_idx].operations[loc.oper_idx]
                .update_serv_oper_type(serv_oper_type),
            None => panic!("Could not find service '{service_name}' to update serv_oper_type"),
        }
    }

    /// get the name of a target defined by proc_idx and oper_idx within this Graph.
    fn get_target(&self, proc_idx: usize, oper_idx: usize) -> String {
        self.0[proc_idx].get_operation_label(oper_idx)
    }

    /// generate a detailled Mermaid diagram, which includes the operations and the outbound calls of each of the services.
    fn mermaid_diagram_full(&self) -> String {
        let mut diagram = Vec::new();
        diagram.push("graph LR".to_string());

        self.0
            .iter()
            .for_each(|p| p.mermaid_add_nodes(&mut diagram));
        self.0
            .iter()
            .for_each(|p| p.mermaid_add_links(&mut diagram, self));

        diagram.join("\n")
    }

    /// Get a compact Mermaid diagram only showing the services, and discarding the detail regarding the actual operation being called.
    fn mermaid_diagram_compact(&self) -> String {
        unimplemented!()
    }

    /// Extract the mermaid diagram based on these imputs
    pub fn mermaid_diagram(&self, compact: bool) -> String {
        if compact {
            self.mermaid_diagram_compact()
        } else {
            self.mermaid_diagram_full()
        }
    }
}
