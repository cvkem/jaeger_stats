use jaeger_stats::{
    read_jaeger_trace_file, test_trace};
use std::{
error::Error,
fs::File,
io::Write};


const SHOW_STDOUT: bool = false;
const INPUT_FILE: &str = "/home/ceesvk/Downloads/372e70a4e259978e.json";
const OUTPUT_FILE: &str = "out.json";


fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{INPUT_FILE}'");
    let jt = read_jaeger_trace_file(INPUT_FILE).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }


    let res =  test_trace(&jt);
    // let s = serde_json::to_string_pretty(&jt)?;
    // println!("Now writing the read Jaeger_trace to {OUTPUT_FILE}");
    // let mut file = File::create(OUTPUT_FILE)?;
    // file.write_all(s.as_bytes())?;

Ok(())
}