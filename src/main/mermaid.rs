use clap::Parser;
use jaeger_stats::{load_viewer, utils, MermaidScope, Metric, TraceDataSet, Viewer};

/// Parsing and analyzin}g Jaeger traces

const EMPTY_ARG: &str = "--";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // file of folder to parse
    //    #[arg(long, default_value_t = String::from("/home/cees/ehome/230717_1122_druk/Stats/cummulative_trace_stats.json"))]
    input: String,

    #[arg(short, long, value_enum, default_value_t = Metric::Count)]
    edge_value: Metric,

    #[arg(short, long)] // "bspc-productinzicht/geefProducten"))]
    service_oper: String,

    #[arg(long, default_value_t = String::from(EMPTY_ARG))]
    call_chain: String,

    #[clap(long, short, action)]
    display: bool,

    #[clap(long, short, action)]
    compact: bool,

    #[arg(long, value_enum, default_value_t = MermaidScope::Full)]
    scope: MermaidScope,
}

fn to_opt_str(s: &str) -> Option<&str> {
    if s != EMPTY_ARG {
        Some(s)
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

    match load_viewer(&args.input) {
        Ok(viewer) => {
            println!(
                "Read the file '{}' as generic.  has time-series {}",
                args.input,
                viewer.is_time_series()
            );
        }
        Err(err) => panic!("Reading '{}' failed with error: {err:?}", args.input),
    }

    match TraceDataSet::from_file(&args.input) {
        Ok(trace_data_set) => {
            println!("Successfully read a TraceDataSet from file {}", args.input);

            println!("The edge_value: {:?}", args.edge_value);

            if args.display {
                let po = trace_data_set.0.get_service_oper_list();
                println!("Service-Operation:\n{}\n\n", get_numbered_lines(po));

                let cc = trace_data_set.0.call_chain_sorted();
                println!("Call-chains:\n{}\n\n", get_numbered_lines(cc));
            }

            let folder = utils::current_folder();
            println!("found folder = {}", folder.to_str().unwrap());
            trace_data_set.write_mermaid_diagram(
                &folder,
                &args.service_oper,
                to_opt_str(&args.call_chain),
                args.edge_value,
                args.scope,
                args.compact,
            )
        }
        Err(err) => panic!("Reading '{}' failed with error: {err:?}", args.input),
    }
}
