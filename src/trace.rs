use std::{
    ffi::OsString,
    iter,
    path::PathBuf};
use crate::{
    span::{build_spans, Spans},
    JaegerTrace,
    raw_jaeger::JaegerItem,
    micros_to_datetime};
use chrono::{
    DateTime,
    Utc};


#[derive(Debug)]
pub struct Trace {
    pub trace_id: String,
    pub root_call: String,
    pub start_dt: DateTime<Utc>,
    pub end_dt: DateTime<Utc>,
    pub duration_micros: u64,
    pub time_to_respond_micros: u64,
    pub missing_span_ids: Vec<String>,
    pub spans: Spans,
}

impl Trace {

    /// build a Trace based upon a JaegerTrace
    pub fn new(jt: &JaegerTrace, idx: usize) -> Self {
        let item = &jt.data[idx];
        let trace_id = item.traceID.to_owned(); 
        //println!(" Found trace: {}", item.traceID);
    
        let (spans, missing_span_ids) = build_spans(item);
    
        let root_call = get_root_call(&spans);
    
    //    let proc_map = build_process_map(item);
    
        let (start_dt, end_dt) = find_full_duration(item);
        let duration_micros = end_dt - start_dt;
        let start_dt = micros_to_datetime(start_dt);
        let end_dt = micros_to_datetime(end_dt);
    
        let time_to_respond_micros = get_response_duration(&spans, item);
    
        Self{trace_id, root_call, start_dt, end_dt,duration_micros, time_to_respond_micros, missing_span_ids, spans}
    }

    /// get the nane of this trace as a CSV-file
    pub fn base_name(&self, folder: &PathBuf) -> OsString {
        let mut folder = folder.clone();
        folder.push(self.trace_id.clone());
        folder.into_os_string()
    }

}

pub fn extract_traces(jt: &JaegerTrace) -> Vec<Trace> {
    let num_traces = jt.data.len();
    (0..num_traces)
        .map(|idx| Trace::new(&jt, idx))
        .collect()
}


fn find_full_duration(ji: &JaegerItem) -> (u64, u64) {
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
fn get_response_duration(spans: &Spans, ji: &JaegerItem) -> u64 {
    // compute start-time based on start_time of earliest span
    let Some(time_to_respond_micros) = iter::zip(spans, &ji.spans)
        .find_map(|(span, jspan)| {
            match span.parent {
                None => Some(jspan.duration),
                Some(_) => None
            } 
        })
    else {
            panic!("Could not find the response duration");
        };
    
    time_to_respond_micros
}


/// get_root_call finds the process and method that is the root-method of the trace.
fn get_root_call(spans: &Spans) -> String {
    // compute start-time based on start_time of earliest span
    let Some(root_call) = spans
        .iter()
        .find_map(|span| {
            match span.parent {
                None => {
                    let proc = span.get_process_str().to_owned();
                    Some(proc + "/" + &span.operation_name)
                },
                _ => None
            }
        })
    else {
        panic!("Could not find an root-span");
    };

    root_call
}


