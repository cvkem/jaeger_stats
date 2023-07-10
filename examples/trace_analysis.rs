use jaeger_stats::process_file_or_folder;
use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
    io::Write};


//const INPUT_FILE: &str = "/home/ceesvk/jaeger/loadTest-prodinz-prodGroep/28adb54b8868eef9.json";

//const INPUT_FILE: &str = "/home/ceesvk/jaeger/loadTest-prodinz-prodGroep/";
const INPUT_FILE: &str = "/home/ceesvk/jaeger/prodinzicht-23-juni-14u/";
const CACHING_PROCESS: &str = "bspc-productinzicht,bspc-partijrolbeheer";

fn main()  {
    let args: Vec<String> = env::args().collect();

    let input_file = if args.len() > 1 {
        args[1].to_owned()
    } else {
        INPUT_FILE.to_owned()
    };

    let caching_processes = if args.len() > 2 {
        args[2].clone()
    } else {
        CACHING_PROCESS.to_owned()
    }.split(",").map(|s| s.to_owned()).collect();

    process_file_or_folder(&input_file, caching_processes);
}