use crate::stats::call_chain::Call;

/// Determine the position of in the diagram, to allow for output of a partial diagram
/// The diagram can be segmented in four regions based on this information:
///  1. Full
///  2.  Centered: view on the selected Service of the current Service-oper combination, and its direct inbound and outbound calls.
///  3.  Inbound: All incoming paths including the selected service;
///  4.  Outbond: The selected service and all of its outbound paths.
/// Combined with the options Compact (true/false) this allows for 8 views.
/// These output formats are needed:
///   * to prevent that views grow to large to be visible (easy zooming in on the relevant parts)
///   * to allow for generation of tailored diagrams for inclusion in architect documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    Unknown, // or do not change position
    Inbound,
    InboundCenter,
    Center,
    OutboundCenter,
    Outbound,
}

impl Position {
    pub fn find_positions(
        from: &Call,
        to: &Call,
        service: &str,
        default_pos: Position,
    ) -> (Self, Self) {
        match (&from.process[..] == service, &to.process[..] == service) {
            (true, true) => (Position::Center, Position::Center),
            (true, false) => (Position::Center, Position::OutboundCenter),
            (false, true) => (Position::InboundCenter, Position::Center),
            (false, false) => (default_pos.clone(), default_pos),
        }
    }

    pub fn check_relevance(self, other: Position) -> Position {
        match (self, other) {
            // retain position if one is unknown
            (Position::Unknown, pos) => pos,
            (pos, Position::Unknown) => pos,
            // Center overrules others
            (_, Position::Center) => Position::Center,
            (Position::Center, _) => Position::Center,
            // Otherwise InboundCenter rules
            (_, Position::InboundCenter) => Position::InboundCenter,
            (Position::InboundCenter, _) => Position::InboundCenter,
            // last position is outboundCenter
            (_, Position::OutboundCenter) => Position::OutboundCenter,
            (Position::OutboundCenter, _) => Position::OutboundCenter,
            // only inbound and outbound remain
            (pos, other_pos) if pos != other_pos => {
                println!(
                    "Strange, this service has position '{:?}' and '{:?}'",
                    pos, other_pos
                );
                pos
            }
            (pos, _) => pos,
        }
    }
}
