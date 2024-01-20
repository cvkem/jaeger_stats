use clap::{Parser, ValueEnum};
use jaeger_stats::{utils, EdgeValue, MermaidScope, StatsRec};
use std::path::Path;

/// Parsing and analyzin}g Jaeger traces

const EMPTY_ARG: &str = "--";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // file of folder to parse
    #[arg(long, default_value_t = String::from("/home/cees/ehome/230717_1122_druk/Stats/cummulative_trace_stats.json"))]
    input: String,

    #[arg(short, long, value_enum, default_value_t = EdgeValue::Count)]
    edge_value: EdgeValue,

    #[arg(short, long, default_value_t = String::from("bspc-productinzicht/geefProducten"))]
    service_oper: String,

    #[arg(long, default_value_t = String::from(EMPTY_ARG))]
    call_chain: String,

    #[clap(long, short, action)]
    display: bool,

    #[clap(long, short, action)]
    compact: bool,

    #[arg(short, long, value_enum, default_value_t = MermaidScope::Full)]
    scope: MermaidScope,
}

fn to_opt_str(s: &String) -> Option<&str> {
    if &s[..] != EMPTY_ARG {
        Some(s.as_str())
    } else {
        None
    }
}

fn get_numbered_lines(data: Vec<String>) -> String {
    let output: Vec<_> = data
        .iter()
        .enumerate()
        .map(|(idx, s)| format!("{idx}: {s}"))
        .collect();
    output.join("\n")
}

fn main() {
    let args = Args::parse();

    let file_path = Path::new(&args.input).to_path_buf().into_os_string(); //get_full_path(base_path, input);
    let traces = StatsRec::read_file(&file_path).unwrap_or_else(|err| {
        panic!(
            "Could not read input-file '{:?}'. Received error: {err:?}",
            file_path
        )
    });

    println!("The edge_value: {:?}", args.edge_value);

    if args.display {
        let po = traces.get_proc_oper_list();
        println!("Service-Operation:\n{}\n\n", get_numbered_lines(po));

        let cc = traces.call_chain_sorted();
        println!("Call-chains:\n{}\n\n", get_numbered_lines(cc));
    }

    let folder = utils::current_folder();
    println!("found folder = {}", folder.to_str().unwrap());
    traces.write_mermaid_diagram(
        &folder,
        &args.service_oper,
        to_opt_str(&args.call_chain),
        args.edge_value,
        args.scope,
        args.compact,
    )
}
