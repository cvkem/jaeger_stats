use clap::Parser;
use jaeger_stats::{
    BestFit, ChartDataParameters, ChartLine, ProcessListItem, Stitched, StitchedDataSet,
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

const PROC_OPER: &str = "bspc-productinzicht/geefProducten";
const CALL_CHAIN_KEY: &str = "retail-gateway/GET:/services/apic-productinzicht/api/producten | retail-gateway//services/apic-productinzicht/api | bspc-productinzicht/geefProducten [Inbound] | bspc-productinzicht/HEAD [Outbound] | bspc-hypotheekaflossingproces/heeftAflossingdetails [Inbound] | bspc-hypotheekaflossingproces/GET [Outbound] | bspc-hypotheekinzicht/zoekHypotheekdetailsPerZekerheid [Inbound] | bspc-hypotheekinzicht/POST [Outbound] | WebSAS/POST [Inbound] | WebSAS/SasFlow [Outbound] | sas/LfiREntrypoint [Inbound] &  [bspc-productinzicht]&  *LEAF*";

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_file = &args.input;

    println!("Reading stitched from '{input_file}'");

    let now = Instant::now();
    let sd = StitchedDataSet::from_file(&input_file).unwrap();
    println!("Elapsed time after load: {}", now.elapsed().as_secs());

    let scope = "full".to_string();
    let mermaid = sd.get_mermaid_diagram(PROC_OPER, Some(CALL_CHAIN_KEY), scope, true);

    println!(
        "The Mermaid-diagram for {}:\n-----------\n\n{}\n\n------------\n",
        PROC_OPER, mermaid
    );
    println!(
        "Elapsed time after handling requests: {}",
        now.elapsed().as_secs()
    );

    Ok(())
}
