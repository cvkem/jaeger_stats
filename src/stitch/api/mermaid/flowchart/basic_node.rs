use super::{super::service_oper_graph::ServiceOperationType, indent::INDENT_STR};

/// A basic node without any nested nodes&
pub struct MermaidBasicNode {
    service: String,
    serv_oper_type: ServiceOperationType,
}

impl MermaidBasicNode {
    pub fn new(service: String, serv_oper_type: ServiceOperationType) -> Self {
        // We can not make service a &str as the string needs to be constructed from service and operation to call new (temporary does not live long enough)
        Self {
            service,
            serv_oper_type,
        }
    }

    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        let indent_str = INDENT_STR.get_indent_str(indent);
        diagram.push(format!(
            "{}{}([{}])",
            indent_str, self.service, self.service
        ));
        if self.serv_oper_type == ServiceOperationType::Emphasized {
            diagram.push(format!("{}style {} fill:#080", indent_str, self.service))
        };
    }
}
