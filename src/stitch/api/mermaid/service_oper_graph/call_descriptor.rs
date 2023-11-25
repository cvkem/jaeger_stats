use crate::stats::call_chain::CallDirection;

use super::{link_type::LinkType, loc::Loc};

/// defines a position in the ProcessConnection, where the first index is the process and the second is the operation.
#[derive(Debug)]
pub struct CallDescriptor {
    pub to_service: usize,
    pub to_oper: usize,
    pub count: Option<f64>,
    pub line_type: LinkType,
    //TODO: is next field needed???
    pub call_direction: CallDirection,
}

impl CallDescriptor {
    pub fn new(loc: Loc, count: Option<f64>, call_direction: CallDirection) -> Self {
        Self {
            to_service: loc.service_idx,
            to_oper: loc.oper_idx,
            count,
            line_type: LinkType::Default,
            call_direction,
        }
    }
}
