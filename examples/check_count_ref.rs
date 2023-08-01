use jaeger_stats::{
    datetime_micros_str, datetime_millis_str, micros_to_datetime, read_jaeger_trace_file,
    JaegerTrace,
};
use std::error::Error;
// use chrono::{
//     DateTime,
//     NaiveDateTime,
//     Utc};

const SHOW_STDOUT: bool = false;
const INPUT_FILE: &str = "/home/ceesvk/Downloads/372e70a4e259978e.json";

fn check_num_references(jt: &JaegerTrace) {
    jt.data.iter().for_each(|ji| {
        ji.spans.iter().for_each(|js| {
            if js.references.len() != 1 {
                println!(
                    "trace_id = {}/{} has {} references",
                    ji.traceID,
                    js.spanID,
                    js.references.len()
                );
            }
        })
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{INPUT_FILE}'");
    let jt = read_jaeger_trace_file(INPUT_FILE).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    check_num_references(&jt);
    Ok(())
}
