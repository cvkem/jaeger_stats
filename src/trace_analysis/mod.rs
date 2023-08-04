use std::{
    path::{Path, PathBuf}
};
use crate::{aux::{Chapter, report},
    stats::{self as crate_stats, CChainEndPointCache}};

use self::read::read_process_file_or_folder;

mod read;
mod write;
mod stats;
mod dedup;


pub fn analyze_file_or_folder(
    path: &Path,
    caching_processes: Vec<String>,
    cc_path: &Path,
) -> PathBuf {
    // Read raw jaeger-traces and process them to clean traces.
    let (traces, num_files, folder) = read_process_file_or_folder(path);

    let folder = folder
        .to_path_buf()
        .canonicalize()
        .expect("Failed to make canonical path. Path probably does not exist!");

    // When joining traces from multiple files we can have duplicates. These should be removed to prevent incorrect statistics
    let traces = dedup::deduplicate(traces);

    // Translate to Extended traces and write the traces to a JSON file
    let traces = crate_stats::build_trace_ext(traces, &folder, num_files, &caching_processes);
    // write the traces
    traces.iter().for_each(|trace| trace.write_trace());

    stats::process_and_fix_traces(
        folder.clone(),
        traces,
        caching_processes,
        cc_path,
        num_files
    );
    folder
}
