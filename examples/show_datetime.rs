use jaeger_stats::{
    datetime_micros_str, datetime_millis_str, micros_to_datetime, read_jaeger_trace_file,
    JaegerTrace,
};
use std::error::Error;

const SHOW_STDOUT: bool = false;
const INPUT_FILE: &str = "/home/ceesvk/Downloads/372e70a4e259978e.json";

fn show_start_times(jt: &JaegerTrace) {
    jt.data.iter().for_each(|ji| {
        ji.spans.iter().for_each(|span| {
            let dt_u64 = span.startTime;

            let dt = micros_to_datetime(dt_u64);

            let dt_millis = datetime_millis_str(dt);
            let dt_micros = datetime_micros_str(dt);

            // Print the newly formatted date and time
            println!("{} (millis)\t\t{} (micros)", dt_millis, dt_micros);
        })
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{INPUT_FILE}'");
    let jt = read_jaeger_trace_file(INPUT_FILE).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    show_start_times(&jt);
    Ok(())
}
