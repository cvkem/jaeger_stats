use clap::Parser;
use jaeger_stats::{set_comma_float, StitchList};
use std::path::Path;

/// Stitching results of different runs of trace_analysis into a single CSV for visualization in Excel
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // List of files to be stitched
    #[arg(short, long, default_value_t = String::from("input.stitch"))]
    stitch_list: String,

    #[arg(short, long, default_value_t = String::from("stitched.csv"))]
    output: String,

    #[arg(short, long, default_value_t = true)]
    comma_float: bool,
}

fn main() {
    let args = Args::parse();

    let stitch_list_path = Path::new(&args.stitch_list);

    set_comma_float(args.comma_float);

    let stitch_list =
        StitchList::read_stitch_list(stitch_list_path).expect("Failed to read stitchlist-file");
    stitch_list.write_stitched_csv(Path::new(&args.output));
}
