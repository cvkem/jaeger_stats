use super::{super::service_oper_graph::LinkType, indent::INDENT_STR};

/// A link between two nodes (either basic node or subgraph)
pub struct MermaidLink {
    src: String,
    target: String,
    value: f64,
    pub link_type: LinkType,
}

impl MermaidLink {
    pub fn new(src: String, target: String, value: f64, link_type: LinkType) -> Self {
        Self {
            src,
            target,
            value,
            link_type,
        }
    }

    pub fn to_diagram(&self, diagram: &mut Vec<String>, indent: usize) {
        let link = match self.link_type {
            LinkType::Emphasized => format!(
                "{}{} ==>|{}| {}",
                INDENT_STR.get_indent_str(indent),
                self.src,
                self.value,
                self.target
            ),
            _ => format!(
                "{}{} -->|{}| {}",
                INDENT_STR.get_indent_str(indent),
                self.src,
                self.value,
                self.target
            ),
        };
        diagram.push(link);
    }
}
