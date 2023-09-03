//! Reading raw json-formatted Jaeger-traces from file
use super::JaegerTrace;
use crate::{
    raw,
    utils::{self, Chapter},
};
use std::{error::Error, ffi::OsStr, fs, path::Path};

/// read a single file and process it to get clean Tcaecs. Returns a set of traces, or an error
fn read_trace_file<T>(
    input_file: &Path,
    process_traces: fn(JaegerTrace) -> Vec<T>,
) -> Result<Vec<T>, Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{}'", input_file.display());
    let jt = raw::read_jaeger_trace_file(input_file).unwrap();

    Ok(process_traces(jt))
}

fn read_trace_folder<T>(
    folder: &Path,
    process_traces: fn(JaegerTrace) -> Vec<T>,
) -> Result<(Vec<T>, i32), Box<dyn Error>> {
    let mut num_files = 0;
    let traces = fs::read_dir(folder)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to extract file-entry");
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            if metadata.is_file() {
                let file_name = path.to_str().expect("path-string").to_owned();
                if file_name.ends_with(".json") {
                    num_files += 1;
                    read_trace_file(&path, process_traces).ok()
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

///Check whether path is a file or folder and read all traces.
pub fn read_process_file_or_folder<T>(
    path: &Path,
    process_traces: fn(JaegerTrace) -> Vec<T>,
) -> (Vec<T>, i32, &Path) {
    utils::report(
        Chapter::Summary,
        format!("Reading all traces from folder: {}", path.display()),
    );
    let (traces, num_files, folder) =
        if path.is_file() && path.extension() == Some(OsStr::new("json")) {
            let traces = read_trace_file(path, process_traces).unwrap();
            (
                traces,
                1,
                path.parent()
                    .expect("Could not extract parent of input_file"),
            )
        } else if path.is_dir() {
            let (traces, num_files) = read_trace_folder(path, process_traces).unwrap();
            (traces, num_files, path)
        } else {
            panic!(
                " Expected file with extention '.json' or folder. Received: '{}' ",
                path.display()
            );
        };
    utils::report(
        Chapter::Summary,
        format!(
            "Read {} traces in total from {} files.",
            traces.len(),
            num_files
        ),
    );

    (traces, num_files, folder)
}

/// change a single Jaeger-trace, possibly containing many traces to a Vector of JaegerTraces each containing a single file.
fn extract_jaeger_traces(jt: JaegerTrace) -> Vec<JaegerTrace> {
    match &jt.errors {
        None => (),
        Some(err) if err.is_empty() => (),
        Some(err) => {
            // TODO:  send this to the report file instead of just console
            println!("Discovered errors: {err:?}");
            ()
        }
    };

    jt.data.into_iter().map(|ji| JaegerTrace::new(ji)).collect()
}

/// read a series of raw Jaeger-traces from a file or a folder
pub fn read_file_or_folder(path: &Path) -> (Vec<JaegerTrace>, i32, &Path) {
    read_process_file_or_folder(path, extract_jaeger_traces)
}
