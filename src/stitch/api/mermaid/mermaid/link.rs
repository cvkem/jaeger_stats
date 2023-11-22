use super::{
    indent::INDENT_STR,
    super::service_oper_graph::{LinkType, ServiceOperationType},
};


/// A link between two nodes (either basic node or subgraph)
pub struct MermaidLink<'a> {
    src: &'a str,
    target: &'a str,
    value: f64,
    pub link_type: LinkType,
}

impl<'a> MermaidLink<'a> {
    pub fn new(src: &'a str, target: &'a str, value: f64, link_type: LinkType) -> Self {
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
    }
}
