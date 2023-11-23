use super::{
    call_descriptor::CallDescriptor, link_type::LinkType, loc::Loc,
    service_oper_type::ServiceOperationType,
};
use crate::stats::call_chain::CallDirection;

/// Used to store outbound
#[derive(Debug)]
pub struct Operation {
    pub oper: String,
    pub call_direction: CallDirection,
    pub serv_oper_type: ServiceOperationType,
    pub calls: Vec<CallDescriptor>,
}

impl Operation {
    pub fn new(oper: String, call_direction: CallDirection) -> Self {
        Self {
            oper,
            call_direction,
            serv_oper_type: ServiceOperationType::Default,
            calls: Vec::new(),
        }
    }

    /// Insert a link to the CallDescriptor 'to', or update it if is exists by adding the count of the 'to' CallDescriptor
    pub fn upsert_link(&mut self, to: CallDescriptor) {
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
    pub fn update_line_type(&mut self, to: Loc, line_type: LinkType) {
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
    pub fn update_serv_oper_type(&mut self, serv_oper_type: ServiceOperationType) {
        self.serv_oper_type = serv_oper_type
    }
}
