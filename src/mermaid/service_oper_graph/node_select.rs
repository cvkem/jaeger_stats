use crate::MermaidScope;

use super::{service::Service, Position};

pub type NodeSelector = fn(&Service) -> bool;

pub fn scope_to_node_selector(scope: MermaidScope) -> NodeSelector {
    match scope {
        MermaidScope::Full => |_service| true,
        MermaidScope::Centered => |service| {
            matches!(
                service.position,
                |super::Position::Center| Position::InboundCenter | Position::OutboundCenter,
            )
        },
        MermaidScope::Inbound => |service| {
            matches!(
                service.position,
                |super::Position::Center| super::Position::Inbound | Position::InboundCenter,
            )
        },
        MermaidScope::Outbound => |service| {
            matches!(
                service.position,
                |super::Position::Center| super::Position::Outbound | Position::OutboundCenter,
            )
        },
    }
}
