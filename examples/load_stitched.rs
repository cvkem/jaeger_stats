use clap::Parser;
use jaeger_stats::Stitched;
use std::{error::Error, time::Instant};

const SHOW_STDOUT: bool = false;

/// Check on references between spans..
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A single json input-file that should be analysed to collect all tags
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_file = &args.input;

    println!("Reading stitched from '{input_file}'");

    let now = Instant::now();
    let data = Stitched::from_json(input_file).unwrap();
    println!("Elapsed time: {}", now.elapsed().as_secs());

    // if SHOW_STDOUT {
    //     println!("{:#?}", data);
    // }

    Ok(())
}
