use clap;
use clap::Parser;
use jaeger_stats::{
    datetime_micros_str, datetime_millis_str, micros_to_datetime, read_jaeger_trace_file,
    JaegerTrace,
};
use std::error::Error;

/// Check on references between spans..
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A single json input-file that should be analysed to collect all tags
    input: String,
}

const SHOW_STDOUT: bool = false;

fn check_num_references(jt: &JaegerTrace) {
    let num_traces = jt.data.len();
    let mut span_without_ref = 0;
    jt.data.iter().for_each(|ji| {
        let mut num_span_wo_ref = 0;
        print!("trace_id = {} spans=[", ji.traceID);
        ji.spans.iter().for_each(|js| {
            if js.references.len() != 1 {
                span_without_ref += 1;
                num_span_wo_ref += 1;
                print!(" {} ", js.spanID);
            }
        });
        if num_span_wo_ref != 1 {
            println!(
                "] Expected exactly ONE root-span. Found {} spans without reference",
                num_span_wo_ref
            );
        } else {
            println!("]");
        }
    });

    println!("Found {span_without_ref} root-spans (spans without a parent reference) on {num_traces}. One such span per trace expected.");
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_file = &args.input;

    println!("Reading a Jaeger-trace from '{input_file}'");
    let jt = read_jaeger_trace_file(input_file).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    check_num_references(&jt);
    Ok(())
}
