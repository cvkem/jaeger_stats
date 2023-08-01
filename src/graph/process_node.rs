use std::collections::HashMap;

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct ProcessNode {
    pub name: String,
    pub ptype: ProcessNodeType,
    /// Operations are the endpoints exposed by the proces (so this is the inbound traffic)
    pub operations: HashMap<String, i32>,
    /// methods are the outbound calls to others
    pub methods: HashMap<String, i32>,
}

impl ProcessNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// for the operation 'oper_name' add count calls.
    pub fn add_operation(&mut self, oper_name: String, count: i32) {
        self.operations
            .entry(oper_name)
            .and_modify(|cnt| *cnt += count)
            .or_insert_with(|| count);
    }
}

pub type ProcessNodes = HashMap<String, ProcessNode>;
