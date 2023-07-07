use jaeger_stats::process_file_or_folder;
use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
    io::Write};


const INPUT_FILE: &str = "/home/ceesvk/jaeger/loadTest-prodinz-prodGroep/28adb54b8868eef9.json";


fn main()  {
    let args: Vec<String> = env::args().collect();

    let input_file = if args.len() > 1 {
        args[1].to_owned()
    } else {
        INPUT_FILE.to_owned()
    };

    let cached_processes = if args.len() > 2 {
        args[2].split(",").map(|s| s.to_owned()).collect()
    } else {
        Vec::new()
    };

    process_file_or_folder(&input_file, cached_processes);
}