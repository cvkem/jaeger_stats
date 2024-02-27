use clap::Parser;
use jaeger_stats::{
    types::{ChartDataParameters, ChartLine, ProcessListItem},
    BestFit, Metric, Stitched, StitchedDataSet, StitchedLine, StitchedSet, TraceScope, Viewer,
};
use log::{error, info};
use serde::Serialize;
use serde_json;
use std::{error::Error, fs, io, time::Instant};

const SHOW_STDOUT: bool = false;

/// Check on references between spans..
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A single json input-file that should be analysed to collect all tags
    input: String,
}

const FILTER_METRIC: Metric = Metric::Rate;
const TRACE_SCOPE: TraceScope = TraceScope::End2end;

const PROCESS: &str = "bspc-productinzicht/geefProducten";
//const PROCESS: &str = "retail-gateway//services/apic-productinzicht/api";

fn dump_proc_list(file_name: &str, proc_list: &Vec<ProcessListItem>) {
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = io::BufWriter::new(f);
    // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human readible)
    match serde_json::to_writer_pretty(writer, proc_list) {
        Ok(()) => (),
        Err(err) => panic!("failed to Serialize !! {err:?}"),
    }
}

fn dump_chart_data(file_name: &str, proc_list: &Option<ChartDataParameters>) {
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = io::BufWriter::new(f);
    // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human-readible)
    match serde_json::to_writer_pretty(writer, proc_list) {
        Ok(()) => (),
        Err(err) => panic!("failed to Serialize !! {err:?}"),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_file = &args.input;

    println!("Reading stitched from '{input_file}'");

    let now = Instant::now();
    let sd = StitchedDataSet::from_file(&input_file).unwrap();
    println!("Elapsed time after load: {}", now.elapsed().as_secs());

    {
        /// Showing Processes
        let proc_list = sd.get_process_list(FILTER_METRIC);

        dump_proc_list("proces_list_mock.json", &proc_list);

        let chart_data = sd.get_service_oper_chart_data(PROCESS, FILTER_METRIC);

        dump_chart_data("charts_mock.json", &chart_data);
    }

    {
        /// Showing Call_chains
        let proc_list = sd.get_call_chain_list(PROCESS, FILTER_METRIC, TRACE_SCOPE, None);

        dump_proc_list("cc_proces_list_mock.json", &proc_list);

        let chart_data = sd.get_call_chain_chart_data(PROCESS, FILTER_METRIC);

        dump_chart_data("cc_charts_mock.json", &chart_data);
    }

    //    println!("{:?}", sd.get_label_list());

    // if SHOW_STDOUT {
    //     println!("{:#?}", data);
    // }
    println!(
        "Elapsed time after handling requests: {}",
        now.elapsed().as_secs()
    );

    Ok(())
}
