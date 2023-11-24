use super::{service::Service, Position};

pub type NodeSelector = fn(&Service) -> bool;

pub fn scope_to_node_selector(scope: &str) -> NodeSelector {
    match scope.to_uppercase().as_str() {
        "FULL" => |_service| true,
        "CENTERED" => |service| match service.position {
            super::Position::Center | Position::InboundCenter | Position::OutboundCenter => true,
            _ => false,
        },
        "INBOUND" => |service| match service.position {
            super::Position::Center | super::Position::Inbound | Position::InboundCenter => true,
            _ => false,
        },
        "OUTBOUND" => |service| match service.position {
            super::Position::Center | super::Position::Outbound | Position::OutboundCenter => true,
            _ => false,
        },
        _ => panic!("Invalid scope for Mermaid: {scope}"),
    }
}
