use crate::{
    StatsMap,
    trace::Trace};
use std::{
    error::Error,
    fs::File,
    io::Write, collections::HashSet,
    path::PathBuf};




pub fn write_string_to_file(filename: &str, data: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// Collect statistics as a string and write it to a textfile in CSV format
pub fn write_stats_to_csv_file(csv_file: &str, stats: &StatsMap) {
    println!("Now writing the trace statistics to {csv_file}");
    let stats_csv_str = stats.to_csv_string();
    write_string_to_file(&csv_file, stats_csv_str);    

}
pub struct TraceExt {
    pub base_name: String,
    pub trace: Trace,
    pub stats: StatsMap,
}

impl TraceExt {

    pub fn new(trace: Trace, folder: &PathBuf, caching_processes: &Vec<String>) -> Self {
        let base_name = trace.base_name(&folder);

        let mut stats = StatsMap::new(caching_processes);
        stats.extend_statistics(&trace, false);
    
        Self{base_name: base_name.into_string().unwrap(), trace, stats}
    }
    
    pub fn get_key(&self) -> String {
        let span = &self.trace.spans[0];
        format!("{}/{}", span.process.as_ref().unwrap().name, span.operation_name).replace(&['/','\\',';'][..], "_")
    }

    pub fn write_trace(&self) {
        let trace_str = format!("{:#?}", self.trace);
        let output_file = format!("{}.txt", self.base_name); 
        println!("Now writing the read Jaeger_trace to {output_file}");
        write_string_to_file(&output_file, trace_str).expect("Failed to write trace (.txt) to file");
    }


    // fn write_stats_csv(&self) {
    //     write_string_to_file(&format!("{}.csv", self.base_name), self.stats.to_csv_string()).expect("Failed to write statistics to file");    
    // }

    // fn fix_tcc_find_matches() {

    // }

    /// Fix the call_chain paths of a trace based on the expected call-chains.
    pub fn fix_trace_call_chain(&mut self, expected_cc: &HashSet<String>) -> bool {
        let exp_cc: Vec<&String> = expected_cc.iter().collect();
        let cc_set = self.stats.call_chain_set();
        let unexpected = cc_set.difference(&expected_cc);

        println!("\nShowing expected:");
        exp_cc.iter()
            .enumerate()
            .for_each(|(idx, cc)|  println!("{idx}: '{cc}'"));

        println!("\nNow trying to find matches:");
        //for cc in unexpected {
        let matched_cc: Vec<_> = unexpected.map(|cc| {

            let matched: Vec<_> = exp_cc
                .iter()
                .filter(|&&x| x.ends_with(cc))
                .collect();
            match matched.len() {
                0 => {
                    if cc.ends_with("*L") {
                        let cc2 = cc.replace("*L", "");
                        let matched: Vec<_> = exp_cc.iter().filter(|&&x| x.ends_with(&cc2)).collect();
                        match matched.len() {
                            0 => {
                                println!("NO-MATCH for '{cc}' as is and as Non-Leaf");
                                None
                            },
                            1 => {
                                println!("MATCHED as NON-leaf");
                                Some(matched[0])
                            },
                            n => {
                                println!("Found '{n}'  matches as Non-leaf and 0 as leaf for '{cc}'");
                                None
                            } 
                        } 
                    } else {
                        println!("NO-MATCH for: '{cc}'");
                        None
                    }
                },
                1 => Some(matched[0]),
                n => {
                    println!("Found {n} matches!! cc= {cc}");
                    None
                }
            }
        })
        .collect();

        if matched_cc.iter().all(|m| m.is_some()) {
            // do the remapping
            println!("!! remapping to be implemented!!");
            true
        } else {
            false
        }
    }

}
