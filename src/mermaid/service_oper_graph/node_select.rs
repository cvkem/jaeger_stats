use crate::mermaid::MermaidScope;

use super::{service::Service, Position};

pub type NodeSelector = fn(&Service) -> bool;

pub fn scope_to_node_selector(scope: MermaidScope) -> NodeSelector {
    match scope {
        MermaidScope::Full => |_service| true,
        MermaidScope::Centered => |service| match service.position {
            super::Position::Center | Position::InboundCenter | Position::OutboundCenter => true,
            _ => false,
        },
        MermaidScope::Inbound => |service| match service.position {
            super::Position::Center | super::Position::Inbound | Position::InboundCenter => true,
            _ => false,
        },
        MermaidScope::Outbound => |service| match service.position {
            super::Position::Center | super::Position::Outbound | Position::OutboundCenter => true,
            _ => false,
        },
    }
}
