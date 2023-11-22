use super::{
    basic_node::MermaidBasicNode,
    super::service_oper_graph::{LinkType, ServiceOperationType},
    node::MermaidNode,
    link::MermaidLink,
    sub_graph::MermaidSubGraph,
};

pub struct Mermaid<'a> {
    nodes: Vec<MermaidNode<'a>>,
    links: Vec<MermaidLink<'a>>,
}

impl<'a> Mermaid<'a> {
    /// add a subgraph, a container that can contain nested simple nodes and subgraphs
    pub fn add_subgraph(&mut self, sg: MermaidSubGraph<'a>) {
        self.nodes.push(MermaidNode::SubGraph(sg))
    }

    /// add a simples node (without any nested nodes)
    pub fn add_node(&mut self, node: MermaidBasicNode<'a>) {
        self.nodes.push(MermaidNode::Node(node))
    }

    /// add a link to this Mermaid diagram
    pub fn add_link(&mut self, link: MermaidLink<'a>) {
        self.links.push(link)
    }

    /// generate a detailled Mermaid diagram, which includes the operations and the outbound calls of each of the services.
    fn mermaid_diagram(&self) -> String {
        let mut diagram = Vec::new();
        diagram.push("graph LR".to_string());

        self.nodes
            .iter()
            .for_each(|node| node.to_diagram(&mut diagram, 1));
        self.links
            .iter()
            .for_each(|link| link.to_diagram(&mut diagram, 1));

        // highligh the emphasized links
        let highlighted: Vec<_> = self
            .links
            .iter()
            .enumerate()
            .filter_map(|(idx, link)| {
                if link.link_type == LinkType::Emphasized {
                    Some(format!("{}", idx + 1))
                } else {
                    None
                }
            })
            .collect();
        diagram.push(format!(
            "linkStyle {} stroke:#ff3,stroke-width:4px,color:red;",
            highlighted.join(",")
        ));

        diagram.join("\n")
    }
}
