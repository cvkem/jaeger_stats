use super::{super::service_oper_graph::LinkType, escape_name::escape_name, indent::INDENT_STR};

/// A link between two nodes (either basic node or subgraph)
pub struct MermaidLink {
    src: String,
    target: String,
    value: Option<f64>,
    pub link_type: LinkType,
}

impl MermaidLink {
    pub fn new(src: String, target: String, value: Option<f64>, link_type: LinkType) -> Self {
        Self {
            src,
            target,
            value,
            link_type,
        }
    }

    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        let value_str = match self.value {
            Some(value) => format!("|{:.0}|", value),
            None => String::new(),
        };
        let esc_src = escape_name(&self.src);
        let esc_target = escape_name(&self.target);
        let link = match self.link_type {
            LinkType::Emphasized => format!(
                "{}{} ==>{} {}",
                INDENT_STR.get_indent_str(indent),
                esc_src,
                value_str,
                esc_target
            ),
            _ => format!(
                "{}{} -->{} {}",
                INDENT_STR.get_indent_str(indent),
                esc_src,
                value_str,
                esc_target
            ),
        };
        diagram.push(link);
    }
}
