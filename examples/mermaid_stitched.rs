use clap::Parser;
use jaeger_stats::{
    BestFit, ChartDataParameters, ChartLine, MermaidScope, ProcessListItem, Stitched,
    StitchedDataSet, StitchedLine, StitchedSet,
};
use log::{error, info};
use serde::Serialize;
use serde_json;
use std::{error::Error, time::Instant};

const SHOW_STDOUT: bool = false;

/// Check on references between spans..
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A single stitched.bincode (or stitched.json) that should be analysed to collect all tags
    input: String,
}

const PROC_OPER: &str = "bspc-productinzicht/geefProducten";
const CALL_CHAIN_KEY: &str = "retail-gateway/GET:/services/apic-productinzicht/api/producten | retail-gateway//services/apic-productinzicht/api | bspc-productinzicht/geefProducten [Inbound] | bspc-productinzicht/HEAD [Outbound] | bspc-hypotheekaflossingproces/heeftAflossingdetails [Inbound] | bspc-hypotheekaflossingproces/GET [Outbound] | bspc-hypotheekinzicht/zoekHypotheekdetailsPerZekerheid [Inbound] | bspc-hypotheekinzicht/POST [Outbound] | WebSAS/POST [Inbound] | WebSAS/SasFlow [Outbound] | sas/LfiREntrypoint [Inbound] &  [bspc-productinzicht]&  *LEAF*";

fn show_mermaid(sd: &StitchedDataSet, scope: &str, compact: bool) {
    let now = Instant::now();

    let mermaid_scope = MermaidScope::try_from(scope).expect("Invalid scope passed");

    let mermaid = sd.get_mermaid_diagram(PROC_OPER, Some(CALL_CHAIN_KEY), mermaid_scope, compact);

    println!("The Mermaid-diagram for {PROC_OPER} and scope '{scope}' and compact={compact}:\n-----------\n\n{mermaid}\n\n------------\n");
    println!(
        "Elapsed time after handling requests: {}",
        now.elapsed().as_secs()
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_file = &args.input;

    println!("Reading stitched from '{input_file}'");

    let now = Instant::now();
    let sd = StitchedDataSet::from_file(&input_file).unwrap();
    println!("Elapsed time after load: {}", now.elapsed().as_secs());

    show_mermaid(&sd, "full", false);
    show_mermaid(&sd, "CenTeRed", false);
    show_mermaid(&sd, "InBoUnd", false);
    show_mermaid(&sd, "outBOUND", false);

    show_mermaid(&sd, "full", true);
    show_mermaid(&sd, "InBoUnd", true);
    show_mermaid(&sd, "outBOUND", true);
    show_mermaid(&sd, "CenTeRed", true);

    println!(
        "Total elapsed time for handling all requests: {}",
        now.elapsed().as_secs()
    );

    Ok(())
}
