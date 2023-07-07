use crate::{
    read_jaeger_trace_file, build_trace, basic_stats, chained_stats, StatsMap,
    trace::Trace, stats::Stats};
use std::{
    error::Error,
    fs::{self, File},
    io::Write};


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

    fn new(input_file: &str) -> Self {
        println!("Reading a Jaeger-trace from '{input_file}'");
        let jt = read_jaeger_trace_file(input_file).unwrap();
    
        if SHOW_STDOUT {
            println!("{:#?}", jt);
        }
    
        let Some(base_name) = input_file.split(".").next() else {
            panic!("Could not split");
        };
    
        let trace = build_trace(&jt);

        let mut stats = StatsMap::new(Vec::new());
        stats.extend_statistics(&trace);
    
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

}



fn process_file(cumm_stats: &mut Option<StatsMap>, input_file: &str) -> Result<(), Box<dyn Error>> {

    let tr = TraceExt::new(input_file);

    let basic_stats = basic_stats(&tr.trace);

    let chained_stats = chained_stats(&tr.trace);

    match cumm_stats {
        Some(cs) => cs.extend_statistics(&tr.trace),
        None => ()
    }

    tr.write_trace();

    tr.write_stats_csv();
    
    Ok(())
}


fn process_json_in_folder(folder: &str, cached_processes: Vec<String>) {
 
//    for entry in fs::read_dir(folder).expect("Failed to read directory") {
    let traces = fs::read_dir(folder)
        .expect("Failed to read directory")
        .into_iter()
        .filter_map(|entry| {
        let entry = entry.expect("Failed to extract file-entry");
        let path = entry.path();

        let metadata = fs::metadata(&path).unwrap();
        if metadata.is_file() {
            let file_name = path.to_str().expect("path-string").to_owned();
            if file_name.ends_with(".json") {
                Some(TraceExt::new(&file_name))
            } else {
                println!("Ignore '{file_name} as it does not have suffix '.json'.");
                None // Not .json file
            }
        } else {
            None  // No file
        }
    });

    let mut cumm_stats = StatsMap::new(cached_processes);

    traces.for_each(|tr| cumm_stats.extend_statistics(&tr.trace));

    write_stats_to_csv_file(&format!("{folder}cummulative_trace_stats.csv"), &cumm_stats);
    // let csv_file = ;
    // println!("Now writing the cummulative trace statistics to {csv_file}");
    // let stats_csv_str = cumm_stats.to_csv_string();
    // write_string_to_file(&csv_file, stats_csv_str);
}


pub fn process_file_or_folder(input_file: &str, cached_processes: Vec<String>)  {

    if input_file.ends_with(".json") {
        process_file(&mut None, &input_file).unwrap();
    } else if input_file.ends_with("/") || input_file.ends_with("\\") {
        process_json_in_folder(&input_file, cached_processes);
    } else {
        panic!(" Expected file with extention '.json'  or folder that ends with '/' (linux) or '\' (windows)");
    }
}