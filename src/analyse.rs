use crate::{
    aux::{write_string_to_file, report, Chapter},
    stats::call_chain::{cchain_filename, CChainEndPointCache},
    raw::read_jaeger_trace_file,
    stats_json::StatsRecJson,
    processed::{extract_traces, Trace},
    stats::traceext::{write_stats_to_csv_file, TraceExt},
    StatsRec,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ffi::OsStr,
    fs,
    io::BufWriter,
    path::{Path, PathBuf},
};

/// read a single file and return a set of traces, or an error
fn read_trace_file(input_file: &Path) -> Result<Vec<Trace>, Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{}'", input_file.display());
    let jt = read_jaeger_trace_file(input_file).unwrap();

    Ok(extract_traces(&jt))
}

fn read_trace_folder(folder: &Path) -> Result<(Vec<Trace>, i32), Box<dyn Error>> {
    let mut num_files = 0;
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
                    num_files += 1;
                    read_trace_file(&path).ok()
                } else {
                    println!("Ignore '{file_name} as it does not have suffix '.json'.");
                    None // Not .json file
                }
            } else {
                None // No file
            }
        })
        .flatten()
        .collect();
    Ok((traces, num_files))
}

/// create the statistics over all traces using the caching_processes
fn create_trace_statistics(
    traces: &Vec<&TraceExt>,
    caching_processes: &Vec<String>,
    num_files: i32,
) -> StatsRec {
    let mut cumm_stats = StatsRec::new(&caching_processes, num_files);
    traces
        .iter()
        .for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false));
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

/// deduplicate all the traces based on traceId and report effect,
fn deduplicate(traces: Vec<TraceExt>) -> Vec<TraceExt> {
    let initial_num = traces.len();

    let mut observed_id = HashSet::new();
    let mut duplicated_ids = Vec::new();

    let traces: Vec<_> = traces
        .into_iter()
        .filter(|tr| {
            let trace_id = &tr.trace.trace_id;
            if observed_id.insert(trace_id.clone()) {
                true // this is a new trace_id
            } else {
                duplicated_ids.push(trace_id.clone());
                false
            }
        })
        .collect();

    let num_duplicates = duplicated_ids.len();
    let remaining = traces.len();
    report(
        Chapter::Summary,
        format!(
            "Removed {num_duplicates}:  So list of {initial_num} traces reduced to {remaining}"
        ),
    );
    report(
        Chapter::Details,
        format!("Removed duplicates: {duplicated_ids:?}"),
    );

    traces
}

fn dump_as_json(file_name: &str, stats: StatsRec) {
    let file_name = file_name.replace(".csv", ".json");
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = BufWriter::new(f);
    let srj: StatsRecJson = stats.into();
    // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human readible)
    match serde_json::to_writer_pretty(writer, &srj) {
        Ok(()) => (),
        Err(err) => panic!("failled to Serialize !! {err:?}"),
    }
}

/// process a vector of traces
fn process_traces(
    folder: PathBuf,
    traces: Vec<Trace>,
    caching_processes: Vec<String>,
    cchain_cache: &mut CChainEndPointCache,
    num_files: i32,
) {
    let folder = folder
        .canonicalize()
        .expect("Failed to make canonical path. Path probably does not exist!");

    let total_traces = traces.len();

    // create a traces folder
    let traces = {
        let trace_folder = extend_create_folder(&folder, "Traces");

        println!("Now generating output for all traces");
        traces
            .into_iter()
            .map(|trace| TraceExt::new(trace, &trace_folder, &caching_processes, num_files))
            .collect::<Vec<_>>()
    };

    let traces = deduplicate(traces);

    //println!("Now writing all traces");
    traces.iter().for_each(|trace| trace.write_trace());

    let stats_folder = extend_create_folder(&folder, "Stats");
    {
        let mut csv_file = stats_folder.clone();
        csv_file.push("cummulative_trace_stats.csv");
        //println!("Writing to file: {:?}", csv_file);
        let traces = traces.iter().collect(); // switch to references
        let cumm_stats = create_trace_statistics(&traces, &caching_processes, num_files);
        write_stats_to_csv_file(&csv_file.to_str().unwrap(), &cumm_stats);
        dump_as_json(&csv_file.to_str().unwrap(), cumm_stats);
    }

    let mut sort_traces = HashMap::new();
    traces.into_iter().for_each(|trace| {
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
                let cumm_stats = create_trace_statistics(&traces.iter().collect(), &caching_processes, num_files);
                // extract call-chains
                let mut cchain_file = cchain_folder.clone();
                cchain_file.push(cchain_filename(&k));
                let cchain_str = cumm_stats.call_chain_str();
                write_string_to_file(&cchain_file.to_str().unwrap(), cchain_str).expect("Failed to write cchain-files.");
                cumm_stats
            } else {
                println!("No complete traces, so we can not produce the call-chain file");
                StatsRec::new(&caching_processes, num_files)
            };

            let part_trace_len = part_traces.len();
            incomplete_traces += part_trace_len;
            if part_traces.len() > 0 {
                let trace_len = traces.len();
                let tot_trace = trace_len + part_trace_len;
                let part_frac = 100.0 * part_trace_len as f64 / tot_trace as f64;
                report(Chapter::Analysis, format!("For end-point (root) '{k}' found {part_trace_len} incomplete out of {tot_trace} traces ({part_frac:.1}%)"));
            }

            // amend/fix traces
            part_traces.iter_mut().for_each(|tr| tr.fix_cchains(cchain_cache));
            // and add these to the statistics
            part_traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false) );
            write_stats_to_csv_file(&csv_file.to_str().unwrap(), &cumm_stats);
            dump_as_json(&csv_file.to_str().unwrap(), cumm_stats);
        });

    println!();
    report(Chapter::Summary, format!("Processed {total_traces} traces covering {num_end_points} end-points  (on average {:.1} trace per end-point).", total_traces as f64/num_end_points as f64));
    report(
        Chapter::Summary,
        format!(
            "Observed {incomplete_traces} incomplete traces, which is {:.1}% of the total",
            100.0 * incomplete_traces as f64 / total_traces as f64
        ),
    );
}

pub fn process_file_or_folder(
    path: &Path,
    caching_processes: Vec<String>,
    cc_path: &Path,
) -> PathBuf {
    report(
        Chapter::Summary,
        format!("Reading all traces from folder: {}", path.display()),
    );
    let (traces, num_files, folder) =
        if path.is_file() && path.extension() == Some(OsStr::new("json")) {
            let traces = read_trace_file(&path).unwrap();
            //let path = Path::new(input_file);
            (
                traces,
                1,
                path.parent()
                    .expect("Could not extract parent of input_file"),
            )
        } else if path.is_dir() {
            let (traces, num_files) = read_trace_folder(&path).unwrap();
            (traces, num_files, path)
        } else {
            panic!(
                " Expected file with extention '.json' or folder. Received: '{}' ",
                path.display()
            );
        };
    report(
        Chapter::Summary,
        format!(
            "Read {} traces in total from {} files.",
            traces.len(),
            num_files
        ),
    );

    let mut cache = CChainEndPointCache::new(cc_path.to_path_buf());
    let folder = folder.to_path_buf();
    process_traces(
        folder.clone(),
        traces,
        caching_processes,
        &mut cache,
        num_files,
    );
    folder
}
