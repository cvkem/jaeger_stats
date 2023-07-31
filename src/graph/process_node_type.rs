use std::collections::HashMap;

#[derive(Default)]
pub enum ProcessNodeType {
    /// represents a prcoess that is directly exposed (via an API gateway), so this is the starting point of a service-process
    EndPoint,
    /// an internal service that supports (public) services (endpoints)
    Intermediate,
    /// a terminal typically receives calls but does not have any outbound calls
    Terminal,
    /// to be determined
    #[default]
    Unknown,
}

struct ProcessNode {
    pub name: String,
    pub ptype: ProcessNodeType,
    /// Operations are the endpoints exposed by the proces (so this is the inbound traffic)
    pub operations: HashMap<String, i32>,
    /// methods are the outbound calls to others
    pub methods: HashMap<String, i32>,
}



