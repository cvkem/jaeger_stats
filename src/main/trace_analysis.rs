use clap::Parser;
use jaeger_stats::{analyze_file_or_folder, set_comma_float, set_tz_offset_minutes, write_report};
use std::path::Path;

/// Parsing and analyzing Jaeger traces

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // file of folder to parse
    input: String,

    #[arg(long)]
    caching_process: Option<String>,

    /// The default source for call-chain information is a sub-folder'CallChain' located in the current folder
    #[arg(short, long, default_value_t = String::from("CallChain/"))]
    call_chain_folder: String,

    #[arg(short = 'z', long, default_value_t = 2*60)]
    timezone_minutes: i64,

    #[arg(short = 'f', long, default_value_t = true)]
    comma_float: bool,

    #[arg(short, long, default_value_t = false)]
    trace_output: bool,

    /// The output-extension determines the output-types are 'json' and 'bincode' (which is also used as the file-extension).
    #[arg(short, long, default_value_t = String::from("json"))]
    output_ext: String,
}

fn main() {
    let args = Args::parse();

    let caching_processes = if let Some(cache_proc) = args.caching_process {
        cache_proc.split(',').map(|s| s.to_owned()).collect()
    } else {
        Vec::new()
    };

    set_tz_offset_minutes(args.timezone_minutes);

    set_comma_float(args.comma_float);

    let mut path = analyze_file_or_folder(
        Path::new(&args.input),
        caching_processes,
        &args.call_chain_folder,
        args.trace_output,
        &args.output_ext,
    );
    path.push("report.txt");
    write_report(path.to_str().unwrap());
}
