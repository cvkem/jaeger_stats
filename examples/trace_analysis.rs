use jaeger_stats::{
    read_jaeger_trace_file, build_trace, basic_stats, chained_stats, StatsMap};
use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
    io::Write};


const SHOW_STDOUT: bool = false;
//const INPUT_FILE: &str = "/home/ceesvk/jaeger/372e70a4e259978e.json";
const INPUT_FILE: &str = "/home/ceesvk/jaeger/loadTest-prodinz-prodGroep/df7e679437c1a05d.json";


fn write_string_to_file(filename: &String, data: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}


fn proces_file(cumm_stats: &mut Option<StatsMap>, input_file: &String) -> Result<(), Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{input_file}'");
    let jt = read_jaeger_trace_file(input_file).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    let Some(base_name) = input_file.split(".").next() else {
        panic!("Could not split");
    };
    let output_file = format!("{base_name}.txt"); 
    let csv_file = format!("{base_name}.csv");

    let spans = build_trace(&jt);

    let basic_stats = basic_stats(&spans);

    let chained_stats = chained_stats(&spans);

    let mut stats = StatsMap::new(Vec::new());
    stats.extend_statistics(&spans);
    match cumm_stats {
        Some(cs) => cs.extend_statistics(&spans),
        None => ()
    }

    let span_str = format!("{spans:#?}");
    println!("Now writing the read Jaeger_trace to {output_file}");
    write_string_to_file(&output_file, span_str);
    // let mut file = File::create(output_file)?;
    // file.write_all(span_str.as_bytes())?;

    println!("Now writing the trace statistics to {csv_file}");
    let stats_csv_str = stats.to_csv_string();
    write_string_to_file(&csv_file, stats_csv_str);

    Ok(())
}


fn process_json_in_folder(folder: &str, cached_processes: Vec<String>) {
    let mut cumm_stats = Some(StatsMap::new(cached_processes));

    for entry in fs::read_dir(folder).expect("Failed to read directory") {
        let entry = entry.expect("Failed to extract file-entry");
        let path = entry.path();

        let metadata = fs::metadata(&path).unwrap();
        if metadata.is_file() {
            let file_name = path.to_str().expect("path-string").to_owned();
            if file_name.ends_with(".json") {
                proces_file(&mut cumm_stats, &file_name).unwrap();
            } else {
                println!("Ignore '{file_name} as it does not have suffix '.json'.");
            }
        }
    }

    if let Some(cumm_stats) = cumm_stats {
        let csv_file = format!("{folder}cummulative_trace_stats.csv");
        println!("Now writing the cummulative trace statistics to {csv_file}");
        let stats_csv_str = cumm_stats.to_csv_string();
        write_string_to_file(&csv_file, stats_csv_str);
    }
}

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

    if input_file.ends_with(".json") {
        proces_file(&mut None, &input_file).unwrap();
    } else if input_file.ends_with("/") || input_file.ends_with("\\") {
        process_json_in_folder(&input_file, cached_processes);
    } else {
        panic!(" Expected file with extention '.json'  or folder that ends with '/' (linux) or '\' (windows)");
    }
}