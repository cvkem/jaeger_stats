/// Read a file and generate JSON again to see if all information was captured completely and correctly in the internal rust-format.
use jaeger_stats::read_jaeger_trace_file;
use std::{error::Error, fs::File, io::Write};

const SHOW_STDOUT: bool = false;
const INPUT_FILE: &str = "/home/ceesvk/Downloads/372e70a4e259978e.json";
const OUTPUT_FILE: &str = "out.json";

fn main() -> Result<(), Box<dyn Error>> {
    println!("Read a file and generate JSON again to see if all information was captured completely and correctly in the internal rust-format.");
    println!("Reading a Jaeger-trace from '{INPUT_FILE}'");
    let jt = read_jaeger_trace_file(INPUT_FILE).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    println!("this method reads the trace from '{INPUT_FILE}' and writes the read data to '{OUTPUT_FILE}'");
    println!("To compare files you need to remove trailing spaces from both input and output via for example:\n\t cat {INPUT_FILE} | sed -e 's/^[ \t]*//' >input.json ");
    println!(" Futhermore we need the next replacements on the input:\n\t\\u003e  -->  '>'\n\t\\u003c --. '<'\n\t\\u0025 --> '&'");
    print!("The full process:\n\tcat{INPUT_FILE} | ");
    println!(
        r"sed -e 's/^[ \t]*//' | sed -e 's/\\u003e/>/g' | sed -e 's/\\u003c/</g' | sed -e 's/\\u0026/\&/g' > input.json"
    );
    println!("\tcat {OUTPUT_FILE} | sed -e 's/^[ \\t]*//'  > output.json");
    println!("\tdiff input.json output.json");

    let s = serde_json::to_string_pretty(&jt)?;
    println!("Now writing the read Jaeger_trace to '{OUTPUT_FILE}'.");
    let mut file = File::create(OUTPUT_FILE)?;
    file.write_all(s.as_bytes())?;

    Ok(())
}
