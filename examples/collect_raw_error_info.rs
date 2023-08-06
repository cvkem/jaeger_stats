use clap;
use clap::Parser;
use jaeger_stats::{read_jaeger_trace_file, JaegerItem, JaegerLog, JaegerSpan, JaegerTrace};
use serde_json::{value::Number, Value};
use std::error::Error;

/// Collecting all span tags from a file and show frequency of occurance.
/// The actual trace-analysis currently only includes a subset of these tags.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A single json input-file that should be analysed to collect all tags
    input: String,
}

const SHOW_STDOUT: bool = false;

#[derive(Debug)]
pub struct TraceErrorInfo {
    pub trace_id: String,
    pub end_point: String,
    pub span_errors: Vec<SpanErrorInfo>,
}

#[derive(Debug)]
pub struct SpanErrorInfo {
    pub process_oper: String,
    pub http_status: u64,
    pub logged_errors: Vec<String>,
}

/// unpack a serde JSON value to an u64 number
fn unpack_serde_u64(v: &Value) -> u64 {
    match v {
        Value::Number(n) => n.as_u64().expect("Could not unpack value to an u64"),
        _ => panic!("Invalid type of the http_code {:?}", v),
    }
}

/// unpack a serde JSON value to a string slice
fn unpack_serde_str(v: &Value) -> &str {
    match v {
        Value::String(s) => &s[..],
        _ => panic!("Invalid type of the http_code {:?}", v),
    }
}

fn span_http_status(js: &JaegerSpan) -> u64 {
    let http_codes: Vec<_> = js
        .tags
        .iter()
        //        .map(|_|  200 as u64)
        .filter_map(|tag| {
            if tag.key == "http.status_code" {
                Some(unpack_serde_u64(&tag.value))
            } else {
                None
            }
        })
        .collect();
    let http_code = match http_codes.len() {
        0 => 200 as u64,
        1 => http_codes[0],
        n => {
            println!(
                "Span {} has {n} http_status_codes {http_codes:?}",
                get_span_label(js)
            );
            //find the first error_code (code > 200) or return 200
            let http_codes: Vec<_> = http_codes.into_iter().filter(|code| *code != 200).collect();
            if http_codes.len() > 0 {
                http_codes[0]
            } else {
                200 as u64
            }
        }
    };
    http_code
}

/// a JeagerLog is a single log-item which has 5 keys i.e.: event, level, logger, message and thread.
/// We only return messages
fn get_error_only(jl: &JaegerLog) -> Option<String> {
    let mut msg = String::new();
    let mut is_error = false;
    jl.fields.iter().for_each(|jt| match &jt.key[..] {
        "level" => match unpack_serde_str(&jt.value) {
            "ERROR" => is_error = true,
            _ => (), // for no error fields we can return Non
        },
        "message" => msg.push_str(unpack_serde_str(&jt.value)),
        _ => (),
    });
    // If we reach this point we should return the message
    match is_error {
        true => Some(msg),
        false => None,
    }
}

fn span_errors(js: &JaegerSpan) -> Vec<String> {
    js.logs.iter().filter_map(|jl| get_error_only(jl)).collect()
}

/// Get the label of the span in the format 'process/operation'.
fn get_span_label(js: &JaegerSpan) -> String {
    format!("{}/{}", js.processID, js.operationName)
}

/// Collects the error_codes over all spans (so the non 200 codes and show the SpanErrorInfo for these spans)
fn collect_span_error_info(ji: &JaegerItem) -> Vec<SpanErrorInfo> {
    ji.spans
        .iter()
        .filter_map(|js| {
            let http_status = span_http_status(js);

            let span_errors = span_errors(js);

            // only return the Spans with an error_code (not 200 Ok)
            match (http_status, span_errors.len()) {
                (200, 0) => None,
                (http_status, _) => Some(SpanErrorInfo {
                    process_oper: get_span_label(js),
                    http_status,
                    logged_errors: span_errors,
                }),
            }
        })
        .collect()
}

fn collect_trace_error_info(jt: &JaegerTrace) -> Vec<TraceErrorInfo> {
    let error_info = jt
        .data
        .iter()
        .filter_map(|ji| {
            let span_error_info = collect_span_error_info(ji);
            if span_error_info.len() > 0 {
                Some(TraceErrorInfo {
                    trace_id: ji.traceID.clone(),
                    end_point: get_span_label(&ji.spans[0]),
                    span_errors: span_error_info,
                })
            } else {
                None
            }
        })
        .collect();
    error_info
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_file = &args.input;

    println!("Reading a Jaeger-trace from '{}'", input_file);
    let jt = read_jaeger_trace_file(input_file).expect("Failed to analyze file");

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    let trace_errors = collect_trace_error_info(&jt);

    println!("TraceErrorInfo:\n{trace_errors:#?}");
    let total_traces = jt.data.len();
    let error_traces = trace_errors.len();
    let perc_failed = (100 * error_traces) as f64 / total_traces as f64;
    println!("\nObserved {error_traces} with errors out of {total_traces}   ({perc_failed:.2}%)");

    Ok(())
}
