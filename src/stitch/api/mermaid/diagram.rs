use super::service_oper_graph::{LinkType, ServiceOperGraph, ServiceOperationType};
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

/// build the ServiceOperationGraph based on the stitched input 'data' and for the selected 'service_oper'.
fn build_serv_oper_graph(data: &Stitched, service_oper: &str) -> ServiceOperGraph {
    let re_service_oper =
        Regex::new(service_oper).expect("Failed to create regex for service_oper");
    let re_so_prefix = Regex::new(&format!("^.*{}", service_oper))
        .expect("Failed to create regex for service_oper_prefix");

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
                sog_cp.0.add_connection(from, to, count);
                // add the counted prefix
                sog_cp.1.add(prefix.as_str(), count);
                sog_cp
            },
        );

    counted_prefix.0.into_iter().for_each(|(k, v)| {
        let cc = CChainStatsKey::parse(&format!("{k} [Unknown] & &")).unwrap();
        std::iter::zip(cc.call_chain.iter(), cc.call_chain.iter().skip(1))
            .for_each(|(s1, s2)| sog.add_connection(s1.clone(), s2.clone(), 0.0));
    });
    // println!("RESULT:\n{:#?}\n\n", sog);

    sog
}

/// Build a daigram for the 'service_oper'  and 'call_chain_key' based on the stitched 'data'.
pub fn get_diagram(
    data: &Stitched,
    service_oper: &str,
    call_chain_key: Option<&str>,
    compact: bool,
) -> String {
    let mut sog = build_serv_oper_graph(data, service_oper);

    // Emphasize the selected path if the call_chain-key is provided
    if let Some(call_chain_key) = call_chain_key {
        let cck = CChainStatsKey::parse(call_chain_key).unwrap();
        std::iter::zip(cck.call_chain.iter(), cck.call_chain.iter().skip(1))
            .for_each(|(from, to)| sog.update_line_type(from, to, LinkType::Emphasized))
    }
    // TODO: implement at the operation level.
    sog.update_service_operation_type(service_oper, ServiceOperationType::Emphasized);
    sog.mermaid_diagram(compact)
}
