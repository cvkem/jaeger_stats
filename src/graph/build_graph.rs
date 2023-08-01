use std::collections::HashMap;
use crate::stats_json::StatsRecJson;
use super::process_node::{ProcessNode, ProcessNodes};



pub fn build_graph(srj: &StatsRecJson) -> ProcessNodes {
    let mut process_nodes = HashMap::new();

    srj.stats.iter()
        .for_each(|(_root_process_key,stat)| {
            stat.call_chain.iter()
                .for_each(|(cc_key, cc_val)| {
                    let count = cc_val.count as i32;
                    cc_key.call_chain.iter()
                        .for_each(|call| {
                            let amend = |pn: &mut ProcessNode| pn.add_operation(call.method.clone(), count);
                            process_nodes
                                .entry(call.process.clone())
                                .and_modify(|pn| {
                                    amend(pn);
                                })
                                .or_insert_with(|| {
                                    let mut pn = ProcessNode::new(call.process.clone());
                                    amend(&mut pn);
                                    pn
                                });
                        })

                })
        });

    process_nodes
}