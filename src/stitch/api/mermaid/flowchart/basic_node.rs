use super::{
    super::service_oper_graph::ServiceOperationType, escape_name::escape_mermaid_label,
    indent::INDENT_STR,
};

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
        let esc_service = escape_mermaid_label(&self.service);
        let the_service = match esc_service.as_ref() {
            Some(esc_service) => esc_service,
            None => &self.service,
        };
        diagram.push(format!(
            "{}{}([\"{}\"])",
            indent_str, the_service, self.service
        ));
        if self.serv_oper_type == ServiceOperationType::Emphasized {
            diagram.push(format!("{}style {} fill:#00802b", indent_str, the_service))
        };
    }
}
