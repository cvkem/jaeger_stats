use super::{
    super::service_oper_graph::LinkType, basic_node::MermaidBasicNode, link::MermaidLink,
    node::MermaidNode, sub_graph::MermaidSubGraph,
};

pub struct Mermaid {
    title: Option<String>,
    nodes: Vec<MermaidNode>,
    links: Vec<MermaidLink>,
}

impl Mermaid {
    pub fn new(title: Option<String>) -> Self {
        Self {
            title,
            nodes: Vec::new(),
            links: Vec::new(),
        }
    }
    /// add a subgraph, a container that can contain nested simple nodes and subgraphs
    pub fn add_subgraph(&mut self, sg: MermaidSubGraph) {
        self.nodes.push(MermaidNode::SubGraph(sg))
    }

    /// add a simples node (without any nested nodes)
    pub fn add_node(&mut self, node: MermaidBasicNode) {
        self.nodes.push(MermaidNode::Node(node))
    }

    /// add a link to this Mermaid diagram
    pub fn add_link(&mut self, link: MermaidLink) {
        self.links.push(link)
    }

    pub fn color_links_statement(&self, link_type: LinkType, color: &str) -> Option<String> {
        // highlight the emphasized links
        let highlighted: Vec<_> = self
            .links
            .iter()
            .enumerate()
            .filter_map(|(idx, link)| {
                if link.link_type == link_type {
                    Some(format!("{}", idx))
                } else {
                    None
                }
            })
            .collect();
        if !highlighted.is_empty() {
            Some(format!(
                "linkStyle {} stroke:{},stroke-width:4px,color:blue;",
                highlighted.join(","),
                color,
            ))
        } else {
            None
        }
    }

    /// generate a detailled Mermaid diagram, which includes the operations and the outbound calls of each of the services.
    pub fn to_diagram(&self) -> String {
        let mut diagram = Vec::new();
        if let Some(title) = self.title.as_ref() {
            diagram.push(format!("---\ntitle: {title}\n---"));
        };
        diagram.push("graph LR".to_string());

        self.nodes
            .iter()
            .for_each(|node| node.to_diagram(&mut diagram, 1));
        self.links
            .iter()
            .for_each(|link| link.to_diagram(&mut diagram, 1));

        // Highlight the emphasized links
        if let Some(line) = self.color_links_statement(LinkType::Emphasized, "#3333ff") {
            diagram.push(line);
        };

        // Highlight the reachable links
        if let Some(line) = self.color_links_statement(LinkType::Reachable, "#99ccff") {
            diagram.push(line);
        };

        diagram.join("\n")
    }
}
