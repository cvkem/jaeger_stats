use crate::{
    processed, raw,
    stats::{self as crate_stats},
    utils,
};

use std::path::{Path, PathBuf};

mod dedup;
mod stats;
mod write;

/// analyze_file_or_folder does the full analysis over a single Jaeger json-file, or a folder that contains a set of json files.
/// All files should be at top-level, so this tool does not inspect sub-folders for json-files (which would not work in fact as sub-folders might contain statistics in json format.).
///
/// /// TODO: a cleaner solution would be based on a chain of iteratos as this:
///    1. Improves readibility code (at least at top level)
///    2. Would make the system less memory intensive as it will become a streaming pipeline which consumes intermediate data as it is produced.
///    3. Would make injection of the processed::extract_traces unnecessary (now needed to process at the bottom of the tree)
/// The challenging part is the stats module where we partition data over two streams.
///  
pub fn analyze_file_or_folder(
    path: &Path,
    caching_processes: Vec<String>,
    cc_path: &str,
    trace_output: bool,
    output_ext: &str,
) -> PathBuf {
    // Read raw jaeger-traces and process them to clean traces.
    let (traces, num_files, folder) =
        raw::read_process_file_or_folder(path, processed::extract_traces);

    let folder = utils::canonicalize_path(folder);
    println!("The folder is '{}'", folder.as_path().display());

    // When joining traces from multiple files we can have duplicates. These should be removed to prevent incorrect statistics
    let traces = dedup::deduplicate(traces);

    // Translate to Extended traces and write the traces to a JSON file
    let traces = crate_stats::build_trace_ext(traces, &folder, num_files, &caching_processes);
    // write the traces

    if trace_output {
        traces.iter().for_each(|trace| trace.write_trace());
    }

    stats::process_and_fix_traces(
        folder.clone(),
        traces,
        caching_processes,
        cc_path,
        num_files,
        output_ext,
    );
    folder
}
