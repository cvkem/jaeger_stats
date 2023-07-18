use jaeger_stats::{
    process_file_or_folder,
    set_comma_float,
    set_tz_offset_minutes,
    write_report};
use std::{
    env,
    path::Path};
use clap;
use clap::Parser;


/// Parsing and analyzing Jaeger traces

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // file of folder to parse
    input: String,

    #[arg(long)]
    caching_process: Option<String>,

    #[arg(short, long, default_value_t = String::from("/home/ceesvk/CallChain/"))]
    call_chain_folder: String,

    #[arg(short, long, default_value_t = 2*60)]
    timezone_minutes: u64,

    #[arg(short='f', long, default_value_t = true)]
    comma_float: bool,
}



//const INPUT_FILE: &str = "/home/ceesvk/jaeger/loadTest-prodinz-prodGroep/28adb54b8868eef9.json";

//const INPUT_FILE: &str = "/home/ceesvk/jaeger/loadTest-prodinz-prodGroep/";
//const INPUT_FILE: &str = "/home/ceesvk/jaeger/prodinzicht-23-juni-14u/";
// const INPUT_FILE: &str = "/home/ceesvk/jaeger/prodInzBatch/";
// const CACHING_PROCESS: &str = "bspc-productinzicht,bspc-partijrolbeheer";
// //const CALL_CHAIN_REPO: &str = "~/CallChain/";
// const CALL_CHAIN_REPO: &str = "/home/ceesvk/CallChain/";

// const TIME_ZONE_MINUTES: u64 = 2*60;

fn main()  {
 
    let args = Args::parse();

    let caching_processes = if let Some(cache_proc) = args.caching_process {
        cache_proc.split(",").map(|s| s.to_owned()).collect()
    } else {
        Vec::new()
    };

    set_tz_offset_minutes(args.timezone_minutes);

    set_comma_float(args.comma_float);

    process_file_or_folder(&Path::new(&args.input), caching_processes, &Path::new(&args.call_chain_folder));
    write_report("report.txt");
}