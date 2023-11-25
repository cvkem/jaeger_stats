use super::service_oper_graph::{LinkType, Position, ServiceOperGraph, ServiceOperationType};
use crate::{stats::CChainStatsKey, Stitched};
use regex::Regex;
use std::collections::HashMap;

struct CountedPrefix(HashMap<String, f64>);

impl CountedPrefix {
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// add a new prefix or increment the existing with 'count'
    fn add(&mut self, prefix: &str, count: f64) {
        self.0
            .entry(prefix.to_owned())
            .and_modify(|v| *v += count)
            .or_insert(count);
    }
}

// split a service out of a service-oper string by spitting at the '/' (or returning the full str if no '/' is present)
fn split_service(service_oper: &str) -> &str {
    service_oper.split("/").next().unwrap()
}

/// Build the ServiceOperationGraph based on the stitched input 'data' and for the selected 'service_oper'.
/// The input 'data' is a dataset of stitched data-point containings all traces though the graph and the statistics for the endpoint of this trace.
/// In this function we reconstruct the original graph by taking the last step of each of the traces that pass through or end in 'service_oper'.
/// The statistic collected is the average number of traces that pass through a node.
/// Some nodes are reachable via multiple paths, in that case the sum is used to aggegate the counts.
///
/// This is a two stage-process.
/// 1. find all paths in 'data' that touch 'service_oper' and construct the graph including the counts (statistics). In this stage we also collect that paths leading to 'service_oper'
/// 2. The (deduplicated) set of all paths leading into 'service_oper' are used to construct all the upstream process-steps. However, we do not have count-statistics for these paths
fn build_serv_oper_graph(data: &Stitched, service_oper: &str) -> ServiceOperGraph {
    let re_service_oper =
        Regex::new(service_oper).expect("Failed to create regex for service_oper");
    let re_so_prefix = Regex::new(&format!("^.*{}", service_oper))
        .expect("Failed to create regex for service_oper_prefix");
    let service = split_service(service_oper);

    // Stage-1: build the downstream graphs and collect the set of incoming paths
    let (mut sog, counted_prefix) = data
        .call_chain
        .iter()
        //            .filter(|(k, _ccd)| k != service_oper) // these are already reported as inbound chains
        .flat_map(|(_k, ccd_vec)| {
            ccd_vec
                .iter()
                //                    .filter(|ccd| all_chains || ccd.is_leaf) // all-chains as we need statistics at each step
                .filter(|ccd| re_service_oper.find(&ccd.full_key).is_some())
                .filter_map(|ccd| {
                    // This closure only takes the last two steps of the path, as this transition is covered by the corresponding dataset
                    let cc = CChainStatsKey::parse(&ccd.full_key).unwrap();
                    if cc.call_chain.len() >= 2 {
                        let skip = cc.call_chain.len() - 2;
                        let mut cc = cc.call_chain.into_iter().skip(skip);
                        let from = cc.next().unwrap();
                        let to = cc.next().unwrap();
                        let count = ccd.data.0.first().and_then(|data| data.data_avg).unwrap();

                        let prefix = re_so_prefix.find(&ccd.full_key).expect("Prefix not found");

                        // and return result
                        Some((prefix, from, to, count))
                    } else {
                        println!("Skipping call-chain as it is consists of a single step 'ccd.full_key' (no link)");
                        None
                    }
                })
        })
        .fold(
            (ServiceOperGraph::new(), CountedPrefix::new()),
            |mut sog_cp, (prefix, from, to, count)| {
                // add the connection to the graph
                sog_cp.0.add_connection(from, to, Some(count), service, Position::Outbound);
                // add the counted prefix
                sog_cp.1.add(prefix.as_str(), count);
                sog_cp
            },
        );

    // Stage 2: amend the graph with the upstream paths (inbound paths)
    counted_prefix.0.into_iter().for_each(|(k, v)| {
        let cc = CChainStatsKey::parse(&format!("{k} [Unknown] & &")).unwrap();
        std::iter::zip(cc.call_chain.iter(), cc.call_chain.iter().skip(1)).for_each(|(s1, s2)| {
            sog.add_connection(s1.clone(), s2.clone(), None, service, Position::Inbound)
        });
    });

    sog
}

/// Mark the selected path in the ServiceOperGraph and return the updated graph
fn mark_selected_call_chain(mut sog: ServiceOperGraph, call_chain_key: &str) -> ServiceOperGraph {
    let cck = CChainStatsKey::parse(call_chain_key).unwrap();
    std::iter::zip(cck.call_chain.iter(), cck.call_chain.iter().skip(1))
        .for_each(|(from, to)| sog.update_line_type(from, to, LinkType::Emphasized));
    sog
}

/// Mark downstream nodes as reachable and do a count of the number of paths reachable over current path up to 'service_oper'
fn mark_and_count_downstream(
    mut sog: ServiceOperGraph,
    service_oper: &str,
    call_chain_key: &str,
) -> ServiceOperGraph {
    sog
}

/// Build a diagram for the 'service_oper'  and 'call_chain_key' based on the stitched 'data'.
pub fn get_diagram(
    data: &Stitched,
    service_oper: &str,
    call_chain_key: Option<&str>,
    scope: String,
    compact: bool,
) -> String {
    let sog = build_serv_oper_graph(data, service_oper);

    let mut sog = if let Some(call_chain_key) = call_chain_key {
        let sog = mark_and_count_downstream(sog, service_oper, call_chain_key);

        // Emphasize the selected path if the call_chain-key is provided
        mark_selected_call_chain(sog, call_chain_key)
    } else {
        sog
    };

    sog.update_service_operation_type(service_oper, ServiceOperationType::Emphasized);
    sog.mermaid_diagram(scope, compact)
}
