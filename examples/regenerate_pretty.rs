use clap;
use clap::Parser;
/// Read a file and generate JSON again to see if all information was captured completely and correctly in the internal rust-format.
use jaeger_stats::read_jaeger_trace_file;
use std::{error::Error, fs::File, io::Write};

/// Read a jaeger json and write it out as pretty-printed json again. A diff of the files should show no differnces.
/// However, this requires that you start with a UTF-8 pretty-printed input-file.(Bulk download on Windows results in UTF-16-LE formatted file in dense/compact format).
/// This tool can also be used for the purpose of pretty-printing a dense-UTF16 json.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A single json input-file that should be analysed to collect all tags
    input: String,
    #[arg(short, long, default_value_t = String::from("output.json"))]
    output: String,
}

const SHOW_STDOUT: bool = false;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_file = &args.input;
    let output = &args.output;

    println!("Read a file and generate JSON again to see if all information was captured completely and correctly in the internal rust-format.");
    println!("Reading a Jaeger-trace from '{input_file}'");
    let jt = read_jaeger_trace_file(input_file).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }

    println!(
        "this method reads the trace from '{input_file}' and writes the read data to '{output}'"
    );
    println!("\nTo compare input and output files you need to remove trailing spaces from both input and output via for example:\n\t cat {input_file} | sed -e 's/^[ \t]*//' >input.json ");
    println!("Removing of trailing whitespace discards difference in indentation-levels and use of tabs (\\t).");
    println!("\nOn linux/uniz the comparison is made via:\n\t diff {input_file} {output}\nwe need the next replacements on the input:\n\t\\u003e  -->  '>'\n\t\\u003c --. '<'\n\t\\u0025 --> '&'");
    print!("The full process:\n\tcat {input_file} | ");
    println!(
        r"sed -e 's/^[ \t]*//' | sed -e 's/\\u003e/>/g' | sed -e 's/\\u003c/</g' | sed -e 's/\\u0026/\&/g' > input.json"
    );
    println!("\tcat {output} | sed -e 's/^[ \\t]*//'  > output.json");
    println!("\tdiff input.json output.json");

    let s = serde_json::to_string_pretty(&jt)?;
    println!("Now writing the read Jaeger_trace to '{output}'.");
    let mut file = File::create(output)?;
    file.write_all(s.as_bytes())?;

    Ok(())
}
