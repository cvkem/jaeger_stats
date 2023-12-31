use super::{link_type::LinkType, loc::Loc};

/// defines a position in the ProcessConnection, where the first index is the process and the second is the operation.
#[derive(Debug)]
pub struct CallDescriptor {
    pub to_service: usize,
    pub to_oper: usize,
    pub count: Option<f64>,
    pub inbound_path_count: Option<f64>,
    pub line_type: LinkType,
}

impl CallDescriptor {
    pub fn new(loc: Loc, count: Option<f64>) -> Self {
        Self {
            to_service: loc.service_idx,
            to_oper: loc.oper_idx,
            count,
            inbound_path_count: None,
            line_type: LinkType::Default,
        }
    }
}
