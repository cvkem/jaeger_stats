use clap::Parser;
use jaeger_stats;
use std::path::Path;

/// Show the Jaeger-traces, or a selection of jaeger-traces, as Pretty-printed JSON in UTF-8 format.

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // file of folder to parse
    input: String,

    /// The default sources is the current folder
    #[arg(short, long, default_value_t = String::from(""))]
    trace_ids: String,

    // /// The default source for call-chain information is a sub-folder'CallChain' located in the current folder
    // #[arg(short, long, default_value_t = String::from("CallChain/"))]
    // call_chain_folder: String,
    #[arg(short = 'z', long, default_value_t = 2*60)]
    timezone_minutes: i64,
}

fn main() {
    let args = Args::parse();

    let (traces, num_files, path) = jaeger_stats::read_file_or_folder(Path::new(&args.input));

    println!("Extracted {} traces from {num_files} files.", traces.len());

    //TODO: deduplication of traces needs to be added here, or in write-traces. However writing traces twice does not harm the proces.

    let num_written = jaeger_stats::write_traces(path, traces, &args.trace_ids);
    println!("Written {num_written} traces to folder Jaeger.")
}
