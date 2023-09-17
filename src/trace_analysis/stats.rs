//! Creating the statistics
use crate::{
    stats::{
        self, call_chain::CChainEndPointCache, file, BasicStatsRec, StatsRec, TraceExt, TraceExtVec,
    },
    utils::{self, Chapter},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// deterimine the cchain folder based upon cc_path if this is an absolute path. If cc_path is a relative path it will be located as a sub-folder of 'folder'.
fn get_cchain_folder(folder: &PathBuf, cc_path: &str) -> PathBuf {
    // Cchain-folder for input and output are set to the same folder.
    utils::report(Chapter::Details, format!("Input for cc_path = {cc_path}"));
    let cc_path = {
        let cc_path_full = Path::new(cc_path).to_path_buf();
        if cc_path_full.is_absolute() {
            cc_path_full
        } else {
            utils::extend_create_folder(&folder, cc_path)
        }
    };
    let cchain_folder = cc_path.to_path_buf();
    // utils::report(
    //     Chapter::Details,
    //     format!("Translates to full cc_path = {}", cc_path.display()),
    // );
    utils::report(
        Chapter::Details,
        format!(
            "Translates to (full) cchain_folder = {}",
            cchain_folder.display()
        ),
    );
    cchain_folder
}

/// create the statistics over all traces using the caching_processes
fn create_trace_statistics(
    traces: &[&TraceExt],
    bsr: &BasicStatsRec,
    rooted_spans_only: bool,
) -> StatsRec {
    let mut cumm_stats = StatsRec::new(bsr.clone());
    traces
        .iter()
        .for_each(|tr| cumm_stats.extend_statistics(&tr.trace, rooted_spans_only));
    cumm_stats
}

/// Try to fix all incomplete traces, while maintaining a cache of call-chains needed for correction, to avoid rereading and parsing the same data
fn fix_traces(cchain_cache: &mut CChainEndPointCache, traces: &mut [TraceExt]) -> usize {
    let mut num_fixes = 0;
    // amend/fix traces
    traces.iter_mut().for_each(|tr| {
        if !tr.trace.missing_span_ids.is_empty() {
            num_fixes += tr.fix_cchains(cchain_cache)
        }
    });

    num_fixes
}

/// write a file showing the statistics over all traces.
fn write_cumulative_trace_stats(
    csv_file: PathBuf,
    traces: &[TraceExt],
    bsr: &BasicStatsRec,
    output_ext: &str,
    rooted_spans_only: bool,
) {
    let num_files = TraceExtVec(&traces).num_files();

    let traces: Vec<_> = traces.iter().collect(); // switch to references
    let cumm_stats = create_trace_statistics(&traces, bsr, rooted_spans_only);
    stats::write_stats_to_csv_file(csv_file.to_str().unwrap(), &cumm_stats);
    file::write_stats(csv_file.to_str().unwrap(), cumm_stats, output_ext);
}

/// Statistics are written per endpoint to the 'Stats' folder, and incomplete traces are corrected (when possible)
/// This involves a multistep process:
///  1. Split traces per end-point such that processing is per endpoint
///  2. For each endpoint:
///    2a. Split traces in two sets, i.e. the complete traces (with a single root defined) and the incomplete traces
///    2b. For complete traces compute the statistics
///    2c. Extract the call-chains form the these statistics and update (initialize) the callChainCache (known call-chains from file are possibly updated with newly discovered information)
///    2d. Use the information from the call-chain-cache to fix the incomplete traces
///    2e. Amend the statistics computed under 2b with the information from the incomplete but corrected traces.
///    2f. Write the statistics to a end-point specific statistics file
///    2g. Collect all traces again for computation of the full-statistics over all end-points AFTER correction of missing spans.
///   The return value of this function is a tuple containign:
///     * the full set of traces
///     * the number of end-points that appaer in these traces
///     * the number of incomplete traces
///     * the number of fixes applied to these incomplete traces (beware that some traces have multiple issues, and not all incomplete traces have been resolved)
fn write_end_point_stats_and_correct_incomplete(
    stats_folder: &PathBuf,
    traces: Vec<TraceExt>, // moving data in and extracting later to prevent the need to copy data. Really needed??
    cchain_cache: &mut CChainEndPointCache,
    mut bsr: BasicStatsRec,
    output_ext: &str,
    rooted_spans_only: bool,
    //    cchain_folder: &PathBuf, // temporary var (TODO: move to caches)
) -> (Vec<TraceExt>, BasicStatsRec) {
    let mut traces_by_endpoint = HashMap::new();
    traces.into_iter().for_each(|trace| {
        let k = trace.get_endpoint_key();
        traces_by_endpoint
            .entry(k)
            .or_insert_with(Vec::new)
            .push(trace);
    });
    // extract call_chain and statistics per call-chain
    let num_end_points = traces_by_endpoint.len();
    let mut num_fixes = 0;
    let mut incomplete_traces_read = 0;
    let mut all_traces = Vec::new();

    traces_by_endpoint.into_iter()
    .for_each(|(k, traces)| {
        let num_files = TraceExtVec(&traces).num_files();
        let mut csv_file = stats_folder.clone();
        csv_file.push(format!("{k}.csv"));
        // The traces that are have 'missing_trace_ids' are the traces that are incomplete, and thus seem to have multiple roots due to the fact
        // that some spans without a parent actually were spans refering a missing span (and not a real root)
        let (traces, mut part_traces): (Vec<_>, Vec<_>) = traces.into_iter().partition(|tr| tr.trace.missing_span_ids.is_empty());
        //TODO: we can produce the call-chains over incomplete traces too if we only include the rooted paths
        let mut cumm_stats = if !traces.is_empty() {
            let cumm_stats = create_trace_statistics(&traces.iter().collect::<Vec<_>>(), &bsr, false);

            cchain_cache.create_update_entry(&k, cumm_stats.call_chain_keys());

            cumm_stats
        } else {
            println!("No complete traces, so we can not produce the call-chain file");
            StatsRec::new(bsr.clone())
        };

        let part_trace_len = part_traces.len();
        incomplete_traces_read += part_trace_len;
        if !part_traces.is_empty() {
            let trace_len = traces.len();
            let tot_trace = trace_len + part_trace_len;
            let part_frac = 100.0 * part_trace_len as f64 / tot_trace as f64;
            utils::report(Chapter::Analysis, format!("For end-point (root) '{k}' found {part_trace_len} incomplete out of {tot_trace} traces ({part_frac:.1}%)"));
        }

        all_traces.extend(traces);

        // amend/fix traces
        let ep_num_fixes = fix_traces(cchain_cache, &mut part_traces);
        num_fixes += ep_num_fixes;

        cumm_stats.num_files = num_files.try_into().unwrap();
        cumm_stats.init_num_incomplete_traces = incomplete_traces_read;
        cumm_stats.num_endpoints = 1;
        cumm_stats.num_fixes = ep_num_fixes;
        cumm_stats.num_incomplete_after_fixes = incomplete_traces_read;  //TODO: to be computed. This estimate is too low.

        // and add these to the statistics
        part_traces.iter().for_each(|tr| cumm_stats.extend_statistics(&tr.trace, rooted_spans_only) );

        stats::write_stats_to_csv_file(csv_file.to_str().unwrap(), &cumm_stats);
        file::write_stats(csv_file.to_str().unwrap(), cumm_stats, output_ext);

        all_traces.extend(part_traces);
    });

    bsr.num_files = TraceExtVec(&all_traces[..]).num_files().try_into().unwrap();
    bsr.num_endpoints = num_end_points;
    bsr.num_fixes = num_fixes;
    bsr.num_incomplete_after_fixes = incomplete_traces_read; //TODO: to be computed. This estimate is too low.

    /// TODO: return the cummulated corrected call-chains
    (all_traces, bsr)
}

/// process a vector of traces
pub fn process_and_fix_traces(
    folder: PathBuf,
    traces: Vec<TraceExt>,
    bsr: BasicStatsRec,
    cc_path: &str,
    output_ext: &str,
) {
    let total_traces = traces.len();

    let stats_folder = utils::extend_create_folder(&folder, "Stats");
    // create the cumulative statistics before correction of call-chain
    // TODO: consider whether this uncorrected version is needed.
    let mut csv_file = stats_folder.clone();
    csv_file.push("cummulative_trace_stats_uncorrected.csv");
    write_cumulative_trace_stats(csv_file, &traces, &bsr, output_ext, false);

    let mut cchain_cache = CChainEndPointCache::new(get_cchain_folder(&folder, cc_path));

    let (all_traces, bsr) = write_end_point_stats_and_correct_incomplete(
        &stats_folder,
        traces,
        &mut cchain_cache,
        bsr, // an updated version is returned as a return value
        output_ext,
        false,
    );

    let mut csv_file = stats_folder.clone();
    csv_file.push("cummulative_trace_stats.csv");
    write_cumulative_trace_stats(csv_file, &all_traces, &bsr, output_ext, false);

    println!();
    utils::report(Chapter::Summary, format!("Processed {total_traces} traces covering {} end-points  (on average {:.1} traces per end-point).",
        bsr.num_endpoints,
        total_traces as f64/bsr.num_endpoints as f64));
    utils::report(
        Chapter::Summary,
        format!(
            "Observed {} incomplete traces, which is {:.1}% of the total",
            bsr.init_num_incomplete_traces,
            100.0 * bsr.init_num_incomplete_traces as f64 / total_traces as f64
        ),
    );
}
