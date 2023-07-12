use crate::{
    read_jaeger_trace_file, basic_stats, chained_stats, StatsMap,
    trace::Trace, stats::Stats};
use std::{
    error::Error,
    fs::{self, File},
    io::Write, collections::HashSet};


const SHOW_STDOUT: bool = false;


fn write_string_to_file(filename: &str, data: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// Collect statistics as a string and write it to a textfile in CSV format
fn write_stats_to_csv_file(csv_file: &str, stats: &StatsMap) {
    println!("Now writing the trace statistics to {csv_file}");
    let stats_csv_str = stats.to_csv_string();
    write_string_to_file(&csv_file, stats_csv_str);    

}
struct TraceExt {
    base_name: String,
    trace: Trace,
    stats: StatsMap,
}

impl TraceExt {

    fn new(input_file: &str, caching_processes: &Vec<String>) -> Self {
        println!("Reading a Jaeger-trace from '{input_file}'");
        let jt = read_jaeger_trace_file(input_file).unwrap();
    
        if SHOW_STDOUT {
            println!("{:#?}", jt);
        }
    
        let Some(base_name) = input_file.split(".").next() else {  // we should collect and drop last segment (the extension)
            panic!("Could not split");
        };
    
        let trace = Trace::new(&jt, 0);

        let mut stats = StatsMap::new(caching_processes);
        stats.extend_statistics(&trace, false);
    
        Self{base_name: base_name.to_owned(), trace, stats}
    }
    

    fn write_trace(&self) {
        let trace_str = format!("{:#?}", self.trace);
        let output_file = format!("{}.txt", self.base_name); 
        println!("Now writing the read Jaeger_trace to {output_file}");
        write_string_to_file(&output_file, trace_str);
    }


    fn write_stats_csv(&self) {
        write_string_to_file(&format!("{}.csv", self.base_name), self.stats.to_csv_string());    
    }

    fn fix_tcc_find_matches() {

    }

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



fn process_file(cumm_stats: &mut Option<StatsMap>, input_file: &str, caching_processes: Vec<String>) -> Result<(), Box<dyn Error>> {

    let tr = TraceExt::new(input_file, &caching_processes);

    let basic_stats = basic_stats(&tr.trace);

    let chained_stats = chained_stats(&tr.trace);

    match cumm_stats {
        Some(cs) => cs.extend_statistics(&tr.trace, false),
        None => ()
    }

    tr.write_trace();

    tr.write_stats_csv();
    
    Ok(())
}


fn process_json_in_folder(folder: &str, caching_processes: Vec<String>) {
 
//    for entry in fs::read_dir(folder).expect("Failed to read directory") {
    let (traces, part_traces): (Vec<_>, Vec<_>) = fs::read_dir(folder)
        .expect("Failed to read directory")
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.expect("Failed to extract file-entry");
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            if metadata.is_file() {
                let file_name = path.to_str().expect("path-string").to_owned();
                if file_name.ends_with(".json") {
                    Some(TraceExt::new(&file_name, &caching_processes))
                } else {
                    println!("Ignore '{file_name} as it does not have suffix '.json'.");
                    None // Not .json file
                }
            } else {
                None  // No file
            }
        })
        .partition(|tr| tr.trace.missing_span_ids.len() == 0);

    let mut cumm_stats = StatsMap::new(&caching_processes);
    if traces.len() == 0 {
        panic!("No complete traces found. Instead found {} partial traces", part_traces.len());
        //traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false));
    }

    // compute statistics over complete traces only
    traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, true) );

    if part_traces.len() > 0 {

        let expected_cc = cumm_stats.call_chain_set();
        let expected_cc_sorted = cumm_stats.call_chain_sorted();

        part_traces
            .into_iter()
            .for_each(|mut tr| {
                if tr.fix_trace_call_chain(&expected_cc) {
                    cumm_stats.extend_statistics(&tr.trace, false);
                } else {
                    println!("Could not fix trace '{}'. Excluded from the analysis",tr.trace.trace_id);
                }
            });
    }
    write_stats_to_csv_file(&format!("{folder}cummulative_trace_stats.csv"), &cumm_stats);
    // let csv_file = ;
    // println!("Now writing the cummulative trace statistics to {csv_file}");
    // let stats_csv_str = cumm_stats.to_csv_string();
    // write_string_to_file(&csv_file, stats_csv_str);
}


pub fn process_file_or_folder(input_file: &str, caching_processes: Vec<String>)  {

    if input_file.ends_with(".json") {
        process_file(&mut None, &input_file, caching_processes).unwrap();
    } else if input_file.ends_with("/") || input_file.ends_with("\\") {
        process_json_in_folder(&input_file, caching_processes);
    } else {
        panic!(" Expected file with extention '.json'  or folder that ends with '/' (linux) or '\' (windows)");
    }
}