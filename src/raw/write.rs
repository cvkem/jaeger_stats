use super::JaegerTrace;
use crate::utils;
use serde_json;
use std::{
    collections::HashSet,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

/// write a single trace as a pretty-printed json to a folder given in 'path' and use the trace_id as the file_base.
fn write_trace(trace_folder: &PathBuf, jt: &JaegerTrace) {
    assert!(!jt.data.is_empty(), "Can not write an empty JaegerTrace");
    let file_path = {
        let file_name = format!("{}.json", jt.data[0].traceID);
        let mut trace_folder = trace_folder.clone();
        trace_folder.push(file_name);
        trace_folder.into_os_string()
    };

    if jt.data.len() != 1 {
        println!("this JaegerTrace contains {} traces, however writing all traces to {} based on the first trace_id.", jt.data.len(), file_path.to_str().expect("Failed to show file_path"))
    }
    let s = serde_json::to_string_pretty(&jt).unwrap_or_else(|err| {
        panic!(
            "Failed to create json for file: '{}'. Received error:\t{err:?}",
            file_path.to_str().expect("Failed to show file_path")
        )
    });

    let mut file = File::create(file_path.clone()).unwrap_or_else(|err| {
        panic!(
            "Failed to create file: '{}'. Received error:\t{err:?}",
            file_path.to_str().expect("Failed to show file_path")
        )
    });
    match file.write_all(s.as_bytes()) {
        Ok(()) => (),
        Err(err) => panic!(
            "Write to path {} failed with error:\t{err:?}",
            file_path.to_str().expect("Failed to show file_path")
        ),
    };
}

/// Write the selected JaegerTraces as JSON to the folder 'Jaeger_trace'. If no selection is provided in 'trace_ids' all traces are written.
pub fn write_traces(folder: &Path, traces: Vec<JaegerTrace>, trace_ids: &str) -> u32 {
    let trace_ids: HashSet<String> = trace_ids
        .split(',')
        .filter(|s| *s != "") // drop the empty strings
        .map(|s| s.to_owned())
        .collect();
    let selected = |jt: &&JaegerTrace| {
        if trace_ids.is_empty() {
            true
        } else {
            trace_ids.get(&jt.data[0].traceID).is_some()
        }
    };

    let traces_folder = utils::extend_create_folder(folder, "Jaeger");

    let mut num_written = 0;
    traces.iter().filter(selected).for_each(|jt| {
        num_written += 1;
        write_trace(&traces_folder, jt)
    });
    num_written
}
