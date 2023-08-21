use clap::Parser;
use jaeger_stats::{set_comma_float, StitchList, Stitched};
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

    #[arg(short, long, default_value_t = String::from("anomalies.csv"))]
    anomalies: String,

    #[arg(short, long, default_value_t = true)]
    comma_float: bool,

    #[arg(short, long, default_value_t = 0)]
    drop_count: usize,
}

fn main() {
    let args = Args::parse();

    let stitch_list_path = Path::new(&args.stitch_list);

    set_comma_float(args.comma_float);

    let stitch_list =
        StitchList::read_stitch_list(stitch_list_path).expect("Failed to read stitchlist-file");
    let stitched = Stitched::build(stitch_list, args.drop_count);

    let path = Path::new(&args.output);
    stitched.write_csv(path);

    println!("Stitched output written to: '{}'", path.display());

    let path = Path::new(&args.anomalies);
    let num_anomalies = stitched.write_anomalies_csv(path);
    if num_anomalies > 0 {
        println!(
            "Detected {num_anomalies}.\n\tFor further information check file '{}'",
            path.display()
        );
    } else {
        println!("NO anomalies detected");
    }
}
