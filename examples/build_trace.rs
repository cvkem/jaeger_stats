use jaeger_stats::{
    read_jaeger_trace_file, build_trace, basic_stats, chained_stats};
use std::{
error::Error,
fs::File,
io::Write};


const SHOW_STDOUT: bool = false;
const INPUT_FILE: &str = "/home/ceesvk/Downloads/372e70a4e259978e.json";
const OUTPUT_FILE: &str = "out.txt";


fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{INPUT_FILE}'");
    let jt = read_jaeger_trace_file(INPUT_FILE).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }


    let spans = build_trace(&jt);

    let basic_stats = basic_stats(&spans);

    let chained_stats = chained_stats(&spans);

    let span_str = format!("BasicStats:\n{basic_stats:#?}\n\nchained Stats:\n{chained_stats:#?}\n\nSpans:\n{spans:#?}");

    println!("Now writing the read Jaeger_trace to {OUTPUT_FILE}");
    let mut file = File::create(OUTPUT_FILE)?;
    file.write_all(span_str.as_bytes())?;

Ok(())
}