use super::{
    super::service_oper_graph::LinkType, escape_name::escape_mermaid_label, indent::INDENT_STR,
};

/// A link between two nodes (either basic node or subgraph)
pub struct MermaidLink {
    src: String,
    target: String,
    value: Option<f64>,
    value2: Option<f64>,
    pub link_type: LinkType,
}

impl MermaidLink {
    pub fn new(
        src: String,
        target: String,
        value: Option<f64>,
        value2: Option<f64>,
        link_type: LinkType,
    ) -> Self {
        Self {
            src,
            target,
            value,
            value2,
            link_type,
        }
    }

    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        let value_str = match (self.value, self.value2) {
            (Some(value), Some(value2)) => format!("|{:.0}/{:.0}|", value, value2),
            (Some(value), None) => format!("|{:.0}|", value),
            (None, _) => String::new(),
        };
        let indent_str = INDENT_STR.get_indent_str(indent);
        let esc_src = escape_mermaid_label(&self.src);
        let esc_src = esc_src.as_ref().unwrap_or(&self.src);
        let esc_target = escape_mermaid_label(&self.target);
        let esc_target = esc_target.as_ref().unwrap_or(&self.target);
        let link = match self.link_type {
            LinkType::Emphasized => {
                format!("{}{} ==>{} {}", indent_str, esc_src, value_str, esc_target)
            }
            _ => format!("{}{} -->{} {}", indent_str, esc_src, value_str, esc_target),
        };
        diagram.push(link);
    }
}
