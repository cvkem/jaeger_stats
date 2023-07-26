use jaeger_stats::{read_stitch_list, StitchList};
use std::{
    path::Path};
use clap;
use clap::Parser;


/// Parsing and analyzing Jaeger traces

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // file of folder to parse
    stitch_list: String,
}




fn main()  {
 
    let args = Args::parse();

    let stitch_list_path = Path::new(&args.stitch_list);

    let stitch_list = read_stitch_list(stitch_list_path).expect("Failed to read stitchlist-file");
    // add the processing
    stitch_list.write_stitched_csv(Path::new("stitched.csv"));
    //println!("{stitch_list:#?}");
}