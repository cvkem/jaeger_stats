use clap::Parser;
use jaeger_stats::{BestFit, Stitched, StitchedLine, StitchedSet};
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

#[derive(Serialize)]
pub struct ProcessListItem {
    pub idx: usize,
    pub name: String,
    pub rank: f64,
}

///TODO: move to stitched_set

const FILTER_METRIC: &str = "rate (avg)";
const PROCESS: &str = "retail-gateway//services/apic-productinzicht/api";

pub fn get_process_list(data: &Stitched, metric: &str) -> Vec<ProcessListItem> {
    let mut proc_list: Vec<_> = data
        .process_operation
        .iter()
        .enumerate()
        .map(|(idx, po)| {
            let line = po.1.get_metric_stitched_line(metric);
            let rank = line.periodic_growth().unwrap_or(-1000.0);

            ProcessListItem {
                idx: idx + 1,
                name: po.0.to_owned(),
                rank,
            }
        })
        .collect();
    proc_list.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());

    proc_list
}

#[derive(Serialize, Debug)]
pub struct ChartLine {
    label: String,
    data: Vec<f64>,
}

#[derive(Serialize, Debug)]
pub struct ChartDataParameters {
    pub title: String,
    pub labels: Vec<String>,
    pub lines: Vec<ChartLine>,
}

pub fn get_chart_data(data: &Stitched, process: &str, metric: &str) -> ChartDataParameters {
    ChartDataParameters {
        title: process.to_owned(),
        labels: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        lines: vec![
            ChartLine {
                label: "Observed".to_string(),
                data: vec![1.2, 2.5, 3.3],
            },
            ChartLine {
                label: "y=1.0 + 1.0*x".to_string(),
                data: vec![2.0, 3.0, 4.0],
            },
            ChartLine {
                label: "y=1.0 *1.05^x".to_string(),
                data: vec![1.0, 2.0, 4.0],
            },
        ],
    }
}

fn dump_proc_list(file_name: &str, proc_list: &Vec<ProcessListItem>) {
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = io::BufWriter::new(f);
    // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human readible)
    match serde_json::to_writer_pretty(writer, proc_list) {
        Ok(()) => (),
        Err(err) => panic!("failed to Serialize !! {err:?}"),
    }
}

fn dump_chart_data(file_name: &str, proc_list: &ChartDataParameters) {
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
    let data = Stitched::from_json(input_file).unwrap();
    println!("Elapsed time: {}", now.elapsed().as_secs());

    let proc_list = get_process_list(&data, FILTER_METRIC);

    dump_proc_list("proces_list_mock.json", &proc_list);

    let chart_data = get_chart_data(&data, PROCESS, FILTER_METRIC);

    dump_chart_data("charts_mock.json", &chart_data);
    // if SHOW_STDOUT {
    //     println!("{:#?}", data);
    // }

    Ok(())
}
