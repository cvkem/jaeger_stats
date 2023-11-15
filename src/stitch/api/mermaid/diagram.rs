use super::service_oper_graph::{LineType, ServiceOperGraph, ServiceOperationType};
use crate::{stats::CChainStatsKey, Stitched};

use regex::Regex;

pub fn get_diagram(
    data: &Stitched,
    service_oper: &str,
    call_chain_key: Option<&str>,
    compact: bool,
) -> String {
    let re_service_oper =
        Regex::new(service_oper).expect("Failed to create regex for service_oper");

    let mut sog = data
        .call_chain
        .iter()
        //            .filter(|(k, _ccd)| k != service_oper) // these are already reported as inbound chains
        .flat_map(|(_k, ccd_vec)| {
            ccd_vec
                .iter()
                //                    .filter(|ccd| all_chains || ccd.is_leaf) // all-chains as we need statistics at each step
                .filter(|ccd| re_service_oper.find(&ccd.full_key).is_some())
                .map(|ccd| {
                    println!("Observed: '{}'", ccd.full_key); // to extract test-data
                    if ccd.full_key == "retail-gateway/GET:/services/apic-productinzicht/api/producten | retail-gateway//services/apic-productinzicht/api | bspc-productinzicht/geefProducten [Inbound] | bspc-productinzicht/HEAD [Outbound] | bspc-hypotheekaflossingproces/heeftAflossingdetails [Inbound] | bspc-hypotheekaflossingproces/GET [Outbound] | bspc-hypotheekinzicht/zoekHypotheekdetailsPerZekerheid [Inbound] | bspc-hypotheekinzicht/POST [Outbound] | WebSAS/POST [Inbound] | WebSAS/SasFlow [Outbound] | sas/LfiREntrypoint [Inbound] &  [bspc-productinzicht]&  *LEAF*" {
                        println!("FOUND relevant path")
                    }
                    let cc = CChainStatsKey::parse(&ccd.full_key).unwrap();
                    let skip = cc.call_chain.len() - 2;
                    let mut cc = cc.call_chain.into_iter().skip(skip);
                    let from = cc.next().unwrap();
                    let to = cc.next().unwrap();
                    let count = ccd.data.0.first().and_then(|data| data.data_avg).unwrap();
                    (from, to, count)
                })
        })
        .fold(ServiceOperGraph::new(), |mut sog, (from, to, count)| {
            sog.add_connection(from, to, count);
            sog
        });

    // println!("RESULT:\n{:#?}\n\n", sog);

    // Emphasize the selected path if the call_chain-key is provided
    if let Some(call_chain_key) = call_chain_key {
        let cck = CChainStatsKey::parse(call_chain_key).unwrap();
        std::iter::zip(cck.call_chain.iter(), cck.call_chain.iter().skip(1))
            .for_each(|(from, to)| sog.update_line_type(from, to, LineType::Emphasized))
    }
    // TODO: implement at the operation level.
    sog.update_service_operation_type(service_oper, ServiceOperationType::Emphasized);
    sog.mermaid_diagram(compact)
}
