use super::span::Spans;
use crate::{
    micros_to_datetime,
    raw::{JaegerItem, JaegerTrace, FILE_TRACKER},
};
use chrono::NaiveDateTime;
use std::{ffi::OsString, path::Path};

#[derive(Debug)]
pub struct Trace {
    pub trace_id: String,
    pub source_file_id: usize,
    pub root_call: String,
    pub start_dt: NaiveDateTime,
    pub end_dt: NaiveDateTime,
    pub duration_micros: i64,
    pub time_to_respond_micros: i64,
    pub missing_span_ids: Vec<String>,
    pub spans: Spans,
}

impl Trace {
    /// build a Trace based upon a JaegerTrace
    pub fn new(jt: &JaegerTrace, idx: usize, source_file_id: usize) -> Self {
        let item = &jt.data[idx];
        let trace_id = item.traceID.to_owned();

        let (spans, missing_span_ids) = Spans::build_spans(item);

        let root_call = get_root_call(&spans);

        let (start_dt, end_dt) = find_full_duration(item);
        let duration_micros = end_dt - start_dt;
        let start_dt = micros_to_datetime(start_dt);
        let end_dt = micros_to_datetime(end_dt);

        let time_to_respond_micros = get_response_duration(&spans, item);

        Self {
            trace_id,
            source_file_id,
            root_call,
            start_dt,
            end_dt,
            duration_micros,
            time_to_respond_micros,
            missing_span_ids,
            spans,
        }
    }

    /// get the nane of this trace as a CSV-file
    pub fn base_name(&self, folder: &Path) -> OsString {
        let mut folder = folder.to_path_buf();
        folder.push(self.trace_id.clone());
        folder.into_os_string()
    }
}

//  returning a closure does not work, as this Fn-closure can not be cloned or copied (for some reason), and that is needed to pass it into another FnMut closure.
// /// Transform a raw JaegerTrace to a vector of Traces. A single JaegerTrace file can contain many traces, and these will be split out.
// pub fn extract_traces(source_file_id: u32) -> Box<dyn Fn(JaegerTrace) -> Vec<Trace>> {
//     Box::new(move |jt| {
//         let num_traces = jt.data.len();
//         (0..num_traces).map(|idx| Trace::new(&jt, idx, source_file_id)).collect()
//     })
// }

/// Transform a raw JaegerTrace to a vector of Traces. A single JaegerTrace file can contain many traces, and these will be split out.
pub fn extract_traces(jt: JaegerTrace) -> Vec<Trace> {
    let num_traces = jt.data.len();
    (0..num_traces)
        .map(|idx| Trace::new(&jt, idx, FILE_TRACKER.lock().unwrap().get_last_idx()))
        .collect()
}

fn find_full_duration(ji: &JaegerItem) -> (i64, i64) {
    // compute start-time based on start_time of earliest span
    let Some(start_dt) = ji.spans
        .iter()
        .map(|jspan| jspan.startTime)
        .min()
    else {
        panic!("Could not find an earliest span");
    };

    // compute start-time based on highest value of start_time+duration over all spans.
    let Some(end_dt) = ji.spans
        .iter()
        .map(|jspan| jspan.startTime + jspan.duration)
        .max()
    else {
        panic!("Could not find an latest span");
    };
    (start_dt, end_dt)
}

/// get_response_duration finds the duration it takes for the root-span to return a response.
/// We iterate over the spans as these have a clear parent-span, while taking the value from the corresponding JaegerItem.
fn get_response_duration(spans: &Spans, ji: &JaegerItem) -> i64 {
    if let Some(root_idx) = spans.root_idx {
        if ji.spans.len() <= root_idx {
            panic!(
                "index {root_idx} does not exist in JaegerItems array with length {}",
                ji.spans.len()
            );
        }
        ji.spans[root_idx].duration
    } else {
        ji.spans[0].duration // beter to find the maximal duration based on early start and last end time
    }
}

/// get_root_call finds the process and method that is the root-method of the trace.
fn get_root_call(spans: &Spans) -> String {
    if let Some(root_idx) = spans.root_idx {
        let root = &spans.items[root_idx];
        format!("{}/{}", root.get_process_str(), root.operation_name)
    } else {
        "_UNKNOWN_".to_string()
    }
}
