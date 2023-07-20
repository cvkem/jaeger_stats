use crate::{
    cchain_cache::CChainEndPointCache,
    report::{Chapter, report},
    StatsRec,
    trace::Trace, cchain_stats::CChainStatsValue};
use std::{
    error::Error,
    fs::File,
    io::Write, collections::HashMap,
    mem,
    path::PathBuf};




pub fn write_string_to_file(filename: &str, data: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// Collect statistics as a string and write it to a textfile in CSV format
pub fn write_stats_to_csv_file(csv_file: &str, stats: &StatsRec) {
    //println!("Now writing the trace statistics to {csv_file}");
    let stats_csv_str = stats.to_csv_string();
    write_string_to_file(&csv_file, stats_csv_str);    

}
pub struct TraceExt {
    pub base_name: String,
    pub trace: Trace,
    pub stats_rec: StatsRec,
}

impl TraceExt {

    pub fn new(trace: Trace, folder: &PathBuf, caching_processes: &Vec<String>) -> Self {
        let base_name = trace.base_name(&folder);

        let mut stats = StatsRec::new(caching_processes);
        stats.extend_statistics(&trace, false);
    
        Self{base_name: base_name.into_string().unwrap(), trace, stats_rec: stats}
    }

    /// Translate the root_call of this trace in an endpoint-key that can be used as base for the file-name to store the call-chains for this endpoint
    pub fn get_endpoint_key(&self) -> String {
        self.trace.root_call.replace(&['/','\\',';',':'][..], "_")
    }

    pub fn write_trace(&self) {
        let trace_str = format!("{:#?}", self.trace);
        let output_file = format!("{}.txt", self.base_name); 
        //println!("Now writing the read Jaeger_trace to {output_file}");
        write_string_to_file(&output_file, trace_str).expect("Failed to write trace (.txt) to file");
    }


    pub fn fix_cchains(&mut self, cchain_cache: &mut CChainEndPointCache) {
        report(Chapter::Details, format!("Trace: {} does have {}", self.base_name, self.trace.missing_span_ids.len()));
        if let Some(expect_cc) = cchain_cache.get_cchain_key(&self.get_endpoint_key()) {
            let new_stats: HashMap<_, _> = mem::take(&mut self.stats_rec.stats)
            .into_iter()
            .map(|(key, mut stats)| {
                let (rooted, mut non_rooted): (Vec<_>, Vec<_>) = stats.call_chain
                    .into_iter()
                    .partition(|(k2, v2)| v2.rooted);

                if non_rooted.len() > 0 {
                    let depths: Vec<_> = non_rooted.iter().map(|(k,v)| v.depth).collect();
                    report(Chapter::Details, format!("For key '{key}'  found {} non-rooted out of {} traces at depths {depths:?}", non_rooted.len(), non_rooted.len() + rooted.len()));
                }

                // fix the non-rooted paths by a rewrite of the key
                let mut fix_failed = 0;
                non_rooted.iter_mut()
                    .for_each(|(k, v)| {
                        if k.remap_callchain(expect_cc) {
                            assert!(!v.rooted);  // should be false
                            v.rooted = true;
                        } else {
                            fix_failed += 1;
                        }});
                if fix_failed > 0 {
                    report(Chapter::Details, format!("Failed to fix {fix_failed} chains out of {} non-rooted chains.", non_rooted.len()));
                }

                let new_call_chain = rooted.into_iter()
                    .chain(non_rooted.into_iter())
                    .fold(HashMap::new(), |mut cc, (k, mut v_new)| {
                        cc.entry(k)
                            .and_modify(|v_curr: &mut CChainStatsValue| {
                                v_curr.count += v_new.count;
                                v_curr.duration_micros.append(&mut v_new.duration_micros);
                            })
                            .or_insert(v_new);
                        cc
                    });
                stats.call_chain = new_call_chain;
                (key, stats) 
            })
            .collect();
            self.stats_rec.stats = new_stats;
        } else {
            println!("Could not find a call-chain for {}", self.trace.root_call);
        }
    }


    // /// Fix the call_chain paths of a trace based on the expected call-chains.
    // pub fn fix_trace_call_chain(&mut self, expected_cc: &HashSet<String>) -> bool {
    //     let exp_cc: Vec<&String> = expected_cc.iter().collect();
    //     let cc_set = self.stats_rec.call_chain_set();
    //     let unexpected = cc_set.difference(&expected_cc);

    //     println!("\nShowing expected:");
    //     exp_cc.iter()
    //         .enumerate()
    //         .for_each(|(idx, cc)|  println!("{idx}: '{cc}'"));

    //     println!("\nNow trying to find matches:");
    //     //for cc in unexpected {
    //     let matched_cc: Vec<_> = unexpected.map(|cc| {

    //         let matched: Vec<_> = exp_cc
    //             .iter()
    //             .filter(|&&x| x.ends_with(cc))
    //             .collect();
    //         match matched.len() {
    //             0 => {
    //                 if cc.ends_with("*L") {
    //                     let cc2 = cc.replace("*L", "");
    //                     let matched: Vec<_> = exp_cc.iter().filter(|&&x| x.ends_with(&cc2)).collect();
    //                     match matched.len() {
    //                         0 => {
    //                             println!("NO-MATCH for '{cc}' as is and as Non-Leaf");
    //                             None
    //                         },
    //                         1 => {
    //                             println!("MATCHED as NON-leaf");
    //                             Some(matched[0])
    //                         },
    //                         n => {
    //                             println!("Found '{n}'  matches as Non-leaf and 0 as leaf for '{cc}'");
    //                             None
    //                         } 
    //                     } 
    //                 } else {
    //                     println!("NO-MATCH for: '{cc}'");
    //                     None
    //                 }
    //             },
    //             1 => Some(matched[0]),
    //             n => {
    //                 println!("Found {n} matches!! cc= {cc}");
    //                 None
    //             }
    //         }
    //     })
    //     .collect();

    //     if matched_cc.iter().all(|m| m.is_some()) {
    //         // do the remapping
    //         println!("!! remapping to be implemented!!");
    //         true
    //     } else {
    //         false
    //     }
    // }

}
