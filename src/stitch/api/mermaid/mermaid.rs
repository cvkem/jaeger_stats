use super::service_oper_graph::{LinkType, ServiceOperationType};

/// A basic node without any nested nodes&
pub struct MermaidBasicNode<'a> {
    service: &'a str,
    serv_oper_type: ServiceOperationType,
}

impl<'a> MermaidBasicNode<'a> {
    pub fn new(service: &'a str, serv_oper_type: ServiceOperationType) -> Self {
        Self{service, serv_oper_type}
    }

    fn to_diagram(&self, diagram: &mut Vec<String>) {
        diagram.push(format!(
            "\t{}([{}])",
            self.service, self.service));
        if self.serv_oper_type == ServiceOperationType::Emphasized {
            diagram.push(format!("\tstyle {} fill:#00f", self.service))
        };    
    }
}

/// A node with an inner structure representing nested nodes.
pub struct MermaidSubGraph<'a> {
    /// service has a plain name, so no embedded '/' is expected.
    service: &'a str,   
    serv_oper_type: ServiceOperationType,
    nodes: Vec<MermaidNode<'a>>,
}

impl<'a> MermaidSubGraph<'a> {
    /// add a subgraph, a container that can contain nested simple nodes and subgraphs 
    pub fn add_subgraph(&mut self, sg: MermaidSubGraph<'a>) {
        self.nodes.push(MermaidNode::SubGraph(sg))
    }

    /// add a simples node (without any nested nodes)
    pub fn add_node(&mut self, node: MermaidBasicNode<'a>) {
        self.nodes.push(MermaidNode::Node(node))
    }

    /// to_diagram
    /// TODO handle indentation
    fn to_diagram(&self, diagram: &mut Vec<String>) {
        diagram.push(format!("\tsubgraph {}", self.service));
        self.nodes.iter().for_each(|node| node.to_diagram(diagram));
        diagram.push("\tend".to_string());

        if self.serv_oper_type == ServiceOperationType::Emphasized {
            diagram.push(format!("\tstyle {} fill:#00f", self.service))
        };
    }
}

/// container to allow for BasicNodes and SubGraphs to be held in the same list
enum MermaidNode<'a> {
    Node(MermaidBasicNode<'a>),
    SubGraph(MermaidSubGraph<'a>),
}

impl<'a> MermaidNode<'a> {
    fn to_diagram(&self, diagram: &mut Vec<String>) {
        match self {
            MermaidNode::Node(node) => node.to_diagram(diagram),
            MermaidNode::SubGraph(sg) => sg.to_diagram(diagram)
        }
    }
}

/// A link between two nodes (either basic node or subgraph)
pub struct MermaidLink<'a> {
    src: &'a str,
    target: &'a str,
    value: f64,
    link_type: LinkType,
}

impl<'a> MermaidLink<'a> {
    pub fn new(src: &'a str, target: &'a str, value: f64, link_type: LinkType) -> Self {
        Self{src, target, value, link_type}
    }

    fn to_diagram(&self, diagram: &mut Vec<String>) {
        let link = match self.link_type {
            LinkType::Emphasized => format!(
                "\t{} ==>|{}| {}",
                self.src, self.value, self.target
            ),
            _ => format!(
                "\t{} -->|{}| {}",
                self.src, self.value, self.target
            ),
        };

    }
}

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
            .for_each(|node| node.to_diagram(&mut diagram));
        self.links
            .iter()
            .for_each(|link| link.to_diagram(&mut diagram));

        // highligh the emphasized links
        let highlighted: Vec<_> = self
            .links
            .iter()
            .enumerate()
            .filter_map(|(idx, link)| if link.link_type == LinkType::Emphasized { Some(format!("{}", idx+1)) } else { None })
            .collect();
        diagram.push(format!(
            "linkStyle {} stroke:#ff3,stroke-width:4px,color:red;",
            highlighted.join(",")
        ));


        diagram.join("\n")
    }
    
}
