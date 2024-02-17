use crate::{StatsRec, types::ProcessList, TraceScope};



/// return a ranked list of processes where rank is based on the periodic-growth of the metric provided.
/// If metric is an empty string the data will be provided in the current order (lexicographic sort.)
pub fn get_process_list(data: &StatsRec, metric: &str) -> ProcessList {
    // let list_size = data.process_operation.len();
    // let proc_list: Vec<_> = data
    //     .process_operation
    //     .iter()
    //     .enumerate()
    //     .map(|(idx, po)| {
    //         // provide a rank based on the reverse of the index, as the highest rank should be in first position.
    //         let rank = if metric.is_empty() {
    //             (list_size - idx) as f64
    //         } else {
    //             get_StatsRec_set_rank(&po.1, metric)
    //         };
    //         let avg_count = get_StatsRec_set_count(&po.1);

    //         ProcessListItem {
    //             idx: (idx + 1) as i64,
    //             key: po.0.to_owned(),
    //             display: po.0.to_owned(),
    //             rank,
    //             avg_count,
    //             chain_type: String::new(),
    //             inbound_idx: 0,
    //         }
    //     })
    //     .collect();

    // reorder_and_renumber(proc_list, metric)
    unimplemented!()
}



// /// get an ordered list of call-chains ranked based on 'metric' that are inbound on a point.
// fn get_call_chain_list_inbound(data: &StatsRec, service_oper: &str, metric: &str) -> ProcessList {
//     let proc_list = match data
//         .call_chain
//         .iter()
//         .filter(|(k, _v)| k == service_oper)
//         .map(|(_k, v)| v)
//         .next()
//     {
//         Some(ccd_vec) => {
//             let list_size = ccd_vec.len();
//             ccd_vec
//                 .iter()
//                 .enumerate()
//                 .map(|(idx, ccd)| {
//                     // provide a rank based on the reverse of the index, as the highest rank should be in first position.
//                     let rank = if metric.is_empty() {
//                         (list_size - idx) as f64
//                     } else {
//                         get_StatsRec_set_rank(&ccd.data, metric)
//                     };

//                     if ccd.inbound_process_key.is_empty() {
//                         println!(
//                             "EMPTY inbound-key on {:?} for {:?}",
//                             ccd.inbound_process_key, ccd.full_key
//                         );
//                     }

//                     let avg_count = get_StatsRec_set_count(&ccd.data);

//                     ProcessListItem {
//                         idx: (idx + 1) as i64,
//                         key: ccd.full_key.to_owned(),
//                         display: ccd.inbound_process_key.to_owned(),
//                         rank,
//                         avg_count,
//                         chain_type: ccd.chain_type().to_owned(),
//                         inbound_idx: 0,
//                     }
//                 })
//                 .collect()
//         }
//         None => {
//             error!("Could not find section for proces_oper = '{service_oper}'");
//             Vec::new()
//         }
//     };

//     reorder_and_renumber(proc_list, metric)
// }

// /// get an ordered list of call-chains ranked based on 'metric' that are end2end process (from end-point to leaf-process of the call-chain).
// fn get_call_chain_list_end2end(
//     data: &StatsRec,
//     service_oper: &str,
//     metric: &str,
//     all_chains: bool,
//     inbound_idx_filter: Option<i64>,
// ) -> ProcessList {
//     let esc_service_oper = regex::escape(service_oper);
//     let re_service_oper =
//         Regex::new(&esc_service_oper).expect("Failed to create regex for service_oper");

//     let inbound_prefix_idx = InboundPrefixIdx::new(data, service_oper);

//     let proc_list = data
//         .call_chain
//         .iter()
//         .filter(|(k, _ccd)| k != service_oper) // these are already reported as inbound chains
//         .flat_map(|(_k, ccd_vec)| {
//             ccd_vec
//                 .iter()
//                 .filter(|ccd| all_chains || ccd.is_leaf)
//                 .filter(|ccd| re_service_oper.find(&ccd.full_key).is_some())
//                 .filter_map(|ccd| {
//                     // provide a rank based on the reverse of the index, as the highest rank should be in first position.
//                     let rank = if metric.is_empty() {
//                         DEFAULT_RANK // will be rewritten before returning this value
//                     } else {
//                         get_StatsRec_set_rank(&ccd.data, metric)
//                     };
//                     let avg_count = get_StatsRec_set_count(&ccd.data);

//                     let inbound_idx = inbound_prefix_idx.get_idx(&ccd.full_key);
//                     if inbound_idx_filter.is_none() || inbound_idx_filter == Some(inbound_idx) {
//                         Some(ProcessListItem {
//                             idx: 0, // will be rewritten
//                             key: ccd.full_key.to_owned(),
//                             display: ccd.inbound_process_key.to_owned(),
//                             rank,
//                             avg_count,
//                             chain_type: ccd.chain_type().to_owned(),
//                             inbound_idx,
//                         })
//                     } else {
//                         None // inbound_idx does not match the filter
//                     }
//                 })
//         })
//         .collect();

//     if metric.is_empty() {
//         rank_lexicographic(proc_list)
//     } else {
//         reorder_and_renumber(proc_list, metric)
//     }
// }

/// get an ordered list of call-chains ranked based on 'metric' that are inbound on a point.
pub fn get_call_chain_list(
    data: &StatsRec,
    service_oper: &str,
    metric: &str,
    scope: TraceScope,
    inbound_idx: Option<i64>,
) -> ProcessList {
    // match scope {
    //     TraceScope::Inbound => get_call_chain_list_inbound(data, service_oper, metric), // default option
    //     TraceScope::End2end => {
    //         get_call_chain_list_end2end(data, service_oper, metric, false, inbound_idx)
    //     }
    //     TraceScope::All => {
    //         get_call_chain_list_end2end(data, service_oper, metric, true, inbound_idx)
    //     }
    // }
    unimplemented!()
}
