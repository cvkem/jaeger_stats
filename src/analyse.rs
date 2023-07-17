use crate::{
    call_chain::cchain_filename,
    cchain_cache::CChainEndPointCache,
    read_jaeger_trace_file,StatsRec,
    trace::{
        Trace, 
        extract_traces},
    traceext::{
        TraceExt,
        write_stats_to_csv_file, write_string_to_file}};
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf}};

/// read a single file and return a set of traces, or an error
fn read_trace_file(input_file: &Path) -> Result<Vec<Trace>, Box<dyn Error>> {

    println!("Reading a Jaeger-trace from '{}'", input_file.display());
    let jt = read_jaeger_trace_file(input_file).unwrap();

    Ok(extract_traces(&jt))
}



fn read_trace_folder(folder: &Path) -> Result<Vec<Trace>, Box<dyn Error>> {
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
                    read_trace_file(&path).ok()
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
fn create_trace_statistics(traces: &Vec<&TraceExt>, caching_processes: &Vec<String>) -> StatsRec {
    let mut cumm_stats = StatsRec::new(&caching_processes);
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

/// process a vector of traces
fn process_traces(folder: PathBuf, traces: Vec<Trace>, caching_processes: Vec<String>, cchain_cache: &mut CChainEndPointCache) {

    folder.canonicalize().expect("Failed to make canonical path. Path probably does not exist!");

    let total_traces = traces.len();

    // create a traces folder
    let traces = {
        let trace_folder = extend_create_folder(&folder,"Traces");

        println!("Now generating output for all traces");
        traces.into_iter()
            .map(|trace| TraceExt::new(trace, &trace_folder, &caching_processes))
            .collect::<Vec<_>>()
    };
    //println!("Now writing all traces");
    traces.iter().for_each(|trace| trace.write_trace());

    let stats_folder = extend_create_folder(&folder, "Stats");
    {
        let mut csv_file = stats_folder.clone();
        csv_file.push("cummulative_trace_stats.csv");
        //println!("Writing to file: {:?}", csv_file);
        let traces = traces.iter().collect(); // switch to references
        let cumm_stats = create_trace_statistics(&traces, &caching_processes);
        write_stats_to_csv_file(&csv_file.to_str().unwrap(), &cumm_stats);
    }

    let mut sort_traces = HashMap::new();
    traces
        .into_iter()
        .for_each(|trace| {
            let k = trace.get_endpoint_key();
            sort_traces.entry(k).or_insert_with(Vec::new).push(trace);
        });
    // extract call_chain and statistics per call-chain
    let num_end_points = sort_traces.len();
    let mut incomplete_traces = 0;
    let cchain_folder = extend_create_folder(&folder, "CallChain");
    sort_traces.into_iter()
        .for_each(|(k, traces)| {
            let mut csv_file = stats_folder.clone();
            csv_file.push(format!("{k}.csv"));
            let (traces, mut part_traces): (Vec<_>, Vec<_>) = traces.into_iter().partition(|tr| tr.trace.missing_span_ids.len() == 0);    
            let mut cumm_stats = if traces.len() > 0 {
                let cumm_stats = create_trace_statistics(&traces.iter().collect(), &caching_processes);
                // extract call-chains
                let mut cchain_file = cchain_folder.clone();
                cchain_file.push(cchain_filename(&k));
                let cchain_str = cumm_stats.call_chain_str();
                write_string_to_file(&cchain_file.to_str().unwrap(), cchain_str).expect("Failed to write cchain-files.");
                cumm_stats
            } else {
                println!("No complete traces, so we can not produce the call-chain file");
                StatsRec::new(&caching_processes)
            };

            let part_trace_len = part_traces.len();
            incomplete_traces += part_trace_len;
            if part_traces.len() > 0 {
                let trace_len = traces.len();
                let tot_trace = trace_len + part_trace_len;
                let part_frac = 100.0 * part_trace_len as f64 / tot_trace as f64;
                println!("For end-point (root) '{k}' found {part_trace_len} incomplete out of {tot_trace} traces ({part_frac:.1}%)");
            }

            // amend/fix traces
            part_traces.iter_mut().for_each(|tr| tr.fix_cchains(cchain_cache));
            // and add these to the statistics
            part_traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false) );
            write_stats_to_csv_file(&csv_file.to_str().unwrap(), &cumm_stats);
        });

        println!("\nProcessed {total_traces} traces covering {num_end_points} end-points  (on average {:.1} per end-point).", total_traces as f64/num_end_points as f64);
        println!("Observed {incomplete_traces}, which is {:.1}% of the total", 100.0 * incomplete_traces as f64/total_traces as f64);
}

    
pub fn process_file_or_folder(path: &Path, caching_processes: Vec<String>, cc_path: &Path)  {
    println!("Reading all traces from folder: {}", path.display());
    let (traces, folder) = if path.is_file() && path.extension() == Some(OsStr::new("json")) {
        let traces = read_trace_file(&path).unwrap();
        //let path = Path::new(input_file);
        (traces, path.parent().expect("Could not extract parent of input_file"))
    } else if path.is_dir() {
        let traces = read_trace_folder(&path).unwrap();
        (traces, path)
    } else {
        panic!(" Expected file with extention '.json'  or folder that ends with '/' (linux) or '\' (windows)");
    };
    println!("....Read {} traces", traces.len());

    let mut cache = CChainEndPointCache::new(cc_path.to_path_buf());
    process_traces(folder.to_path_buf(), traces, caching_processes, &mut cache);

}