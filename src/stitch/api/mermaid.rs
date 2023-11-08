use super::super::Stitched;
use crate::stats::CChainStatsKey;

use regex::Regex;

pub fn get_diagram(data: &Stitched, proc_oper: &str, call_chain_key: Option<&str>) -> String {
    let re_proc_oper = Regex::new(proc_oper).expect("Failed to create regex for proc_oper");

    let proc_list: Vec<_> = data
        .call_chain
        .iter()
        //            .filter(|(k, _ccd)| k != proc_oper) // these are already reported as inbound chains
        .flat_map(|(_k, ccd_vec)| {
            ccd_vec
                .iter()
                //                    .filter(|ccd| all_chains || ccd.is_leaf) // all-chains as we need statistics at each step
                .filter(|ccd| re_proc_oper.find(&ccd.full_key).is_some())
                .filter_map(|ccd| {
                    let cc = CChainStatsKey::parse(&ccd.full_key).unwrap();
                    let skip = cc.call_chain.len() - 3;
                    let mut cc = cc.call_chain.into_iter().skip(skip);
                    let prev_receive_call = cc.next().unwrap();
                    let send_call = cc.next().unwrap();
                    let receive_call = cc.next().unwrap();
                    let count = ccd.data.0.first().and_then(|data| data.data_avg).unwrap();
                    Some((
                        ccd.full_key.to_owned(),
                        prev_receive_call,
                        send_call,
                        receive_call,
                        count,
                    ))
                })
        })
        .collect();

    proc_list
        .into_iter()
        //            .map(|(k, send_call, receive_call, count)| format!("{}\n\t{:?} -> {:?}  cnt={}", k, send_call, receive_call, count))
        .map(|(_k, prev_receive_call, send_call, receive_call, count)| {
            format!(
                "{} -> {} -> {}  cnt={}",
                prev_receive_call.to_string(),
                send_call.to_string(),
                receive_call.to_string(),
                count
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
