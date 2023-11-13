use super::service_oper_graph::{LineType, ServiceOperGraph};
use crate::{stats::CChainStatsKey, Stitched};

use regex::Regex;

pub fn get_diagram(
    data: &Stitched,
    proc_oper: &str,
    call_chain_key: Option<&str>,
    compact: bool,
) -> String {
    let re_proc_oper = Regex::new(proc_oper).expect("Failed to create regex for proc_oper");

    let mut pog = data
        .call_chain
        .iter()
        //            .filter(|(k, _ccd)| k != proc_oper) // these are already reported as inbound chains
        .flat_map(|(_k, ccd_vec)| {
            ccd_vec
                .iter()
                //                    .filter(|ccd| all_chains || ccd.is_leaf) // all-chains as we need statistics at each step
                .filter(|ccd| re_proc_oper.find(&ccd.full_key).is_some())
                .map(|ccd| {
                    let cc = CChainStatsKey::parse(&ccd.full_key).unwrap();
                    let skip = cc.call_chain.len() - 2;
                    let mut cc = cc.call_chain.into_iter().skip(skip);
                    let from = cc.next().unwrap();
                    let to = cc.next().unwrap();
                    let count = ccd.data.0.first().and_then(|data| data.data_avg).unwrap();
                    (from, to, count)
                })
        })
        .fold(ServiceOperGraph::new(), |mut pog, (from, to, count)| {
            pog.add_connection(from, to, count);
            pog
        });

    // Emphasize the selected path if the call_chain-key is provided
    if let Some(call_chain_key) = call_chain_key {
        let cck = CChainStatsKey::parse(call_chain_key).unwrap();
        std::iter::zip(cck.call_chain.iter(), cck.call_chain.iter().skip(1))
            .for_each(|(from, to)| pog.update_line_type(from, to, LineType::Emphasized))
    }
    pog.mermaid_diagram(compact)
}
