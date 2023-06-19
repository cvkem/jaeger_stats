use jaeger_stats::{
    read_jaeger_trace_file, build_trace, basic_stats, chained_stats, StatsMap};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write};


const SHOW_STDOUT: bool = false;
//const INPUT_FILE: &str = "/home/ceesvk/Downloads/372e70a4e259978e.json";
const INPUT_FILE: &str = "/home/ceesvk/Downloads/4cd5114ce8c5c387.json";
const OUTPUT_FILE: &str = "out.txt";



fn proces_file(input_file: &String) -> Result<(), Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{input_file}'");
    let jt = read_jaeger_trace_file(input_file).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    let Some(base_name) = input_file.split(".").next() else {
        panic!("Could not split");
    };
    let output_file = format!("{base_name}.txt"); 

    let spans = build_trace(&jt);

    let basic_stats = basic_stats(&spans);

    let chained_stats = chained_stats(&spans);

    let mut stats = StatsMap::new();
    stats.extend_statistics(&spans);

    let span_str = format!("StatsMap:\n{stats:#?}\n\nBasicStats:\n{basic_stats:#?}\n\nchained Stats:\n{chained_stats:#?}\n\nSpans:\n{spans:#?}");

    println!("Now writing the read Jaeger_trace to {output_file}");
    let mut file = File::create(output_file)?;
    file.write_all(span_str.as_bytes())?;

    Ok(())
}


fn process_json_in_folder(folder: &str) {
    for entry in fs::read_dir(folder).expect("Failed to read directory") {
        let entry = entry.expect("Failed to extract file-entry");
        let path = entry.path();

        let metadata = fs::metadata(&path).unwrap();
        if metadata.is_file() {
            let file_name = path.to_str().expect("path-string").to_owned();
            if file_name.ends_with(".json") {
                proces_file(&file_name).unwrap();
            } else {
                println!("Ignore '{file_name} as it does not have suffix '.json'.");
            }
        }
    }
}

fn main()  {
    let args: Vec<String> = env::args().collect();

    let input_file = if args.len() > 1 {
        args[1].to_owned()
    } else {
        INPUT_FILE.to_owned()
    };

    if input_file.ends_with(".json") {
        proces_file(&input_file).unwrap();
    } else {
        process_json_in_folder(&input_file);
    }
}