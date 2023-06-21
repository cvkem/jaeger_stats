use std::iter;
use crate::{
    process_map::build_process_map,
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
    pub start_dt: DateTime<Utc>,
    pub end_dt: DateTime<Utc>,
    pub duration_micros: u64,
    pub time_to_respond_micros: u64,
    pub spans: Spans,
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

/// get_response_duration finds the duration needed before the root-span returns a response.
fn get_response_duration(ji: &JaegerItem) -> u64 {
    // compute start-time based on start_time of earliest span
    let Some(time_to_respond_micros) = ji.spans
        .iter()
        .find_map(|jspan| {
            if jspan.references.len() == 0 {
                // return duration of span that has no parents.
                Some(jspan.duration)
            } else {
                None
            }
        })
    else {
        panic!("Could not find an root-span");
    };

    time_to_respond_micros
}

pub fn build_trace(jt: &JaegerTrace) -> Trace {
    let item = &jt.data[0];
    let trace_id = item.traceID.to_owned(); 
    println!(" Found trace: {}", item.traceID);

    let spans = build_spans(jt);

    let proc_map = build_process_map(item);

    let (start_dt, end_dt) = find_full_duration(item);
    let duration_micros = end_dt - start_dt;
    let start_dt = micros_to_datetime(start_dt);
    let end_dt = micros_to_datetime(end_dt);

    let time_to_respond_micros = get_response_duration(item);

    Trace{trace_id, start_dt, end_dt,duration_micros, time_to_respond_micros, spans}
}

