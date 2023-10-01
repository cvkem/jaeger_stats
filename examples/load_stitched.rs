use clap::Parser;
use jaeger_stats::{
    get_call_chain_chart_data, get_call_chain_list, get_label_list, get_proc_oper_chart_data,
    get_process_list, BestFit, ChartDataParameters, ChartLine, ProcessListItem, Stitched,
    StitchedLine, StitchedSet,
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

const FILTER_METRIC: &str = "rate (avg)";
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
    // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human readible)
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
    let data = match Stitched::from_json(input_file) {
        Ok(stitched) => stitched,
        Err(err) => {
            error!("Failed to load file '{input_file}'");
            panic!("Failure during load of file: {input_file}");
        }
    };
    println!("Elapsed time after load: {}", now.elapsed().as_secs());

    {
        /// Showing Processes
        let proc_list = get_process_list(&data, FILTER_METRIC);

        dump_proc_list("proces_list_mock.json", &proc_list);

        let chart_data = get_proc_oper_chart_data(&data, PROCESS, FILTER_METRIC);

        dump_chart_data("charts_mock.json", &chart_data);
    }

    {
        /// Showing Call_chains
        let proc_list = get_call_chain_list(&data, PROCESS, FILTER_METRIC);

        dump_proc_list("cc_proces_list_mock.json", &proc_list);

        let chart_data = get_call_chain_chart_data(&data, PROCESS, PROCESS, FILTER_METRIC);

        dump_chart_data("cc_charts_mock.json", &chart_data);
    }

    println!("{:?}", get_label_list(&data));

    // if SHOW_STDOUT {
    //     println!("{:#?}", data);
    // }
    println!(
        "Elapsed time after handling requests: {}",
        now.elapsed().as_secs()
    );

    Ok(())
}
