//! Creating the statistics
use crate::{
    aux::{extend_create_folder, report, write_string_to_file, Chapter},
    stats::{
        call_chain::{cchain_filename, CChainEndPointCache},
        json::dump_as_json,
        write_stats_to_csv_file, StatsRec, TraceExt,
    },
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// create the statistics over all traces using the caching_processes
fn create_trace_statistics(
    traces: &[&TraceExt],
    caching_processes: &[String],
    num_files: i32,
) -> StatsRec {
    let mut cumm_stats = StatsRec::new(caching_processes, num_files);
    traces
        .iter()
        .for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false));
    cumm_stats
}

/// process a vector of traces
pub fn process_and_fix_traces(
    folder: PathBuf,
    traces: Vec<TraceExt>,
    caching_processes: Vec<String>,
    cc_path: &Path,
    num_files: i32,
) {
    let total_traces = traces.len();

    let stats_folder = extend_create_folder(&folder, "Stats");
    {
        let mut csv_file = stats_folder.clone();
        csv_file.push("cummulative_trace_stats.csv");
        //println!("Writing to file: {:?}", csv_file);
        let traces: Vec<_> = traces.iter().collect(); // switch to references
        let cumm_stats = create_trace_statistics(&traces, &caching_processes, num_files);
        write_stats_to_csv_file(csv_file.to_str().unwrap(), &cumm_stats);
        dump_as_json(csv_file.to_str().unwrap(), cumm_stats);
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
            let (traces, mut part_traces): (Vec<_>, Vec<_>) = traces.into_iter().partition(|tr| tr.trace.missing_span_ids.is_empty());
            let mut cumm_stats = if !traces.is_empty() {
                let cumm_stats = create_trace_statistics(&traces.iter().collect::<Vec<_>>(), &caching_processes, num_files);
                // extract call-chains
                let mut cchain_file = cchain_folder.clone();
                cchain_file.push(cchain_filename(&k));
                let cchain_str = cumm_stats.call_chain_str();
                write_string_to_file(cchain_file.to_str().unwrap(), cchain_str).expect("Failed to write cchain-files.");
                cumm_stats
            } else {
                println!("No complete traces, so we can not produce the call-chain file");
                StatsRec::new(&caching_processes, num_files)
            };

            let part_trace_len = part_traces.len();
            incomplete_traces += part_trace_len;
            if !part_traces.is_empty() {
                let trace_len = traces.len();
                let tot_trace = trace_len + part_trace_len;
                let part_frac = 100.0 * part_trace_len as f64 / tot_trace as f64;
                report(Chapter::Analysis, format!("For end-point (root) '{k}' found {part_trace_len} incomplete out of {tot_trace} traces ({part_frac:.1}%)"));
            }

            let mut cchain_cache = CChainEndPointCache::new(cc_path.to_path_buf());

            // amend/fix traces
            part_traces.iter_mut().for_each(|tr| tr.fix_cchains(&mut cchain_cache));
            // and add these to the statistics
            part_traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, false) );
            write_stats_to_csv_file(csv_file.to_str().unwrap(), &cumm_stats);
            dump_as_json(csv_file.to_str().unwrap(), cumm_stats);
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
