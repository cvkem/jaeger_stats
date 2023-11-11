use super::proc_oper_graph::ProcOperGraph;
use crate::{stats::CChainStatsKey, Stitched};

use regex::Regex;

pub fn get_diagram(
    data: &Stitched,
    proc_oper: &str,
    call_chain_key: Option<&str>,
    compact: bool,
) -> String {
    let re_proc_oper = Regex::new(proc_oper).expect("Failed to create regex for proc_oper");

    let pog = data
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
                    let send_call = cc.next().unwrap();
                    let receive_call = cc.next().unwrap();
                    let count = ccd.data.0.first().and_then(|data| data.data_avg).unwrap();
                    (send_call, receive_call, count)
                })
        })
        .fold(
            ProcOperGraph::new(),
            |mut pog, (send_call, receive_call, count)| {
                pog.add(send_call, receive_call, count);
                pog
            },
        );
    pog.mermaid_diagram(compact)
}
