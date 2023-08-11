use clap::Parser;
use jaeger_stats::{build_graph, StatsRecJson};
use std::ffi::OsString;

/// Stitching results of different runs of trace_analysis into a single CSV for visualization in Excel
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // List of files to be stitched
    #[arg(default_value_t = String::from("input.json"))]
    input: String,

    #[arg(short, long, default_value_t = String::from("stitched.csv"))]
    output: String,
    // #[arg(short, long, default_value_t = true)]
    // comma_float: bool,
}

fn main() {
    let args = Args::parse();

    let stats_rec_path = OsString::from(&args.input);

    //    set_comma_float(args.comma_float);

    let data = StatsRecJson::read_file(&stats_rec_path).expect("Failed to read JSON-file");
    // add the processing
    let graph = build_graph(&data);

    println!("{graph:#?}");
}
