use crate::{
    read_jaeger_trace_file,StatsMap,
    trace::{
        Trace, 
        extract_traces},
    traceext::{
        TraceExt,
        write_stats_to_csv_file}};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf}};

/// read a single file and return a set of traces, or an error
fn read_trace_file(input_file: &str) -> Result<Vec<Trace>, Box<dyn Error>> {

    println!("Reading a Jaeger-trace from '{input_file}'");
    let jt = read_jaeger_trace_file(input_file).unwrap();

    Ok(extract_traces(&jt))
}


fn read_trace_folder(folder: &str) -> Result<Vec<Trace>, Box<dyn Error>> {
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
                    read_trace_file(&file_name).ok()
                } else {
                    println!("Ignore '{file_name} as it does not have suffix '.json'.");
                    None // Not .json file
                }
            } else {
                None  // No file
            }
        })
        .flatten()
        .collect();
        Ok(traces)
    }

/// create the statistics over all traces using the caching_processes and write them to the file
fn create_trace_statistics(traces: &Vec<&TraceExt>, caching_processes: &Vec<String>) -> StatsMap {
    let mut cumm_stats = StatsMap::new(&caching_processes);
    traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false) );
    cumm_stats
}


/// create a sub-folder if it does not exist yet and return the path to this sub-folder
fn extend_create_folder(folder: &PathBuf, subfolder: &str) -> PathBuf {
    let mut ext_folder = folder.clone();
    ext_folder.push(subfolder);
    if !ext_folder.is_dir() {
        fs::create_dir(ext_folder.clone()).expect("failed to create folder");
    }
    ext_folder
}

/// process a vector of traces.
fn process_traces(folder: PathBuf, traces: Vec<Trace>, caching_processes: Vec<String>) {

    folder.canonicalize().expect("Failed to make canonical path. Path probably does not exist!");

    // create a traces folder
    let traces = {
        let trace_folder = extend_create_folder(&folder,"Traces");

        println!("Now generating output for all traces");
        traces.into_iter()
            .map(|trace| TraceExt::new(trace, &trace_folder, &caching_processes))
            .collect::<Vec<_>>()
    };
    println!("Now writing all traces");
    traces.iter().for_each(|trace| trace.write_trace());

    let stats_folder = extend_create_folder(&folder, "Stats");
    {
        let mut csv_file = stats_folder.clone();
        csv_file.push("cummulative_trace_stats.csv");
        println!("Writing to file: {:?}", csv_file);
        let (traces, part_traces): (Vec<_>, Vec<_>) = traces.iter().partition(|tr| tr.trace.missing_span_ids.len() == 0);    
        let mut cumm_stats = create_trace_statistics(&traces, &caching_processes);
        // extract call-chains
        // amend/fix traces
        traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false) );
        write_stats_to_csv_file(&csv_file.to_str().unwrap(), &cumm_stats);
    }

    let mut sort_traces = HashMap::new();
    traces
        .into_iter()
        .for_each(|trace| {
            let k = trace.get_key();
            sort_traces.entry(k).or_insert_with(Vec::new).push(trace);
        });

    sort_traces.into_iter()
        .for_each(|(k, traces)| {
            let mut csv_file = stats_folder.clone();
            csv_file.push(format!("{k}.csv"));
            let traces = traces.iter().collect(); // switch to references
            let cumm_stats = create_trace_statistics(&traces, &caching_processes);
            write_stats_to_csv_file(&csv_file.to_str().unwrap(), &cumm_stats);
        });

    //     let stats = StatsMap::new(^caching_process);
    //     stats.extend_statistics(trace, false);
    //     let filename = trace.txt_file_name(&mut folder);
    //     let trace_str = format!("{:#?}", trace);
    //     write_string_to_file(&filename, trace_str);

    // });



    // let (traces, part_traces): (Vec<_>, Vec<_>) = traces.into_iter().partition(|tr| tr.missing_span_ids.len() == 0);    
    // let mut cumm_stats = StatsMap::new(&caching_processes);
    // if traces.len() == 0 {
    //     panic!("No complete traces found. Instead found {} partial traces", part_traces.len());
    //     //traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false));
    // }

    // // compute statistics over complete traces only
    // traces.iter().for_each(|tr| cumm_stats.extend_statistics(tr, true) );

    // if part_traces.len() > 0 {

    //     let expected_cc = cumm_stats.call_chain_set();
    //     let expected_cc_sorted = cumm_stats.call_chain_sorted();

    //     part_traces
    //         .into_iter()
    //         .for_each(|mut tr| {
    //             if tr.fix_trace_call_chain(&expected_cc) {
    //                 cumm_stats.extend_statistics(&tr, false);
    //             } else {
    //                 println!("Could not fix trace '{}'. Excluded from the analysis",tr.trace_id);
    //             }
    //         });
    // }
    // write_stats_to_csv_file(&format!("{}cummulative_trace_stats.csv", folder.display()), &cumm_stats);
}


pub fn process_file_or_folder(input_file: &str, caching_processes: Vec<String>)  {

    let (traces, folder) = if input_file.ends_with(".json") {
        let traces = read_trace_file(&input_file).unwrap();
        let path = Path::new(input_file);
        (traces, path.parent().expect("Could not extract parent of input_file"))
    } else if input_file.ends_with("/") || input_file.ends_with("\\") {
        let traces = read_trace_folder(&input_file).unwrap();
        (traces, Path::new(input_file))
    } else {
        panic!(" Expected file with extention '.json'  or folder that ends with '/' (linux) or '\' (windows)");
    };

    process_traces(folder.to_path_buf(), traces, caching_processes);

}