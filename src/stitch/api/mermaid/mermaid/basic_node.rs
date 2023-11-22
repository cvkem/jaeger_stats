use super::{
    indent::INDENT_STR,
    super::service_oper_graph::{LinkType, ServiceOperationType},
};

/// A basic node without any nested nodes&
pub struct MermaidBasicNode<'a> {
    service: &'a str,
    serv_oper_type: ServiceOperationType,
}

impl<'a> MermaidBasicNode<'a> {
    pub fn new(service: &'a str, serv_oper_type: ServiceOperationType) -> Self {
        Self {
            service,
            serv_oper_type,
        }
    }

    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        diagram.push(format!(
            "{}{}([{}])",
            INDENT_STR.get_indent_str(indent),
            self.service,
            self.service
        ));
        if self.serv_oper_type == ServiceOperationType::Emphasized {
            diagram.push(format!("\tstyle {} fill:#00f", self.service))
        };
    }
}
