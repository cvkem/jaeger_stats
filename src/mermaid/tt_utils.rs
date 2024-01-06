use super::service_oper_graph::{LinkType, ServiceOperGraph};
use crate::stats::CChainStatsKey;
use regex;

// split a service out of a service-oper string by spitting at the '/' (or returning the full str if no '/' is present)
pub fn split_service(service_oper: &str) -> &str {
    service_oper.split("/").next().unwrap()
}

pub fn get_call_chain_prefix(service_oper: &str, call_chain_key: &str) -> String {
    let esc_service_oper = regex::escape(service_oper);
    let prefix =
        regex::Regex::new(&format!("^.*{}", esc_service_oper)).expect("Failed to create a regex");
    match prefix.find(call_chain_key) {
        Some(result) => result.as_str().to_owned(),
        None => panic!(
            "Could not find service-oper '{service_oper}' in call-chain-key '{call_chain_key}'"
        ),
    }
}

/// Mark the selected path in the ServiceOperGraph and return the updated graph
pub fn mark_selected_call_chain(
    mut sog: ServiceOperGraph,
    call_chain_key: &str,
) -> ServiceOperGraph {
    let cck = CChainStatsKey::parse(call_chain_key).unwrap();
    std::iter::zip(cck.call_chain.iter(), cck.call_chain.iter().skip(1))
        .for_each(|(from, to)| sog.update_line_type(from, to, LinkType::Emphasized));
    sog
}
