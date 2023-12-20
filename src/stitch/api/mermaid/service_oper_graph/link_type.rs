#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LinkType {
    Default,
    Reachable,
    // CurrentReach,
    Emphasized,
}

impl LinkType {
    pub fn merge(self, other: LinkType) -> LinkType {
        match (self, other) {
            //            (a, b) if a == b => a,
            (LinkType::Default, LinkType::Default) => LinkType::Default,
            (LinkType::Reachable, LinkType::Default) => LinkType::Reachable,
            (LinkType::Default, LinkType::Reachable) => LinkType::Reachable,
            (LinkType::Reachable, LinkType::Reachable) => LinkType::Reachable,
            (LinkType::Emphasized, _) => LinkType::Emphasized,
            (_, LinkType::Emphasized) => LinkType::Emphasized,
        }
    }
}
// link-style example: linkStyle 37,10,22 stroke:#ff3,stroke-width:4px,color:red;
