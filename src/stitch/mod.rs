//! Stich does read a series of analysis files defined in Stitch and stitches these files together for a trend analysis.
//! So basically stitch does a transposition of data from a columnar format (per moment in time) to a row-based format where each row represents a specific metric
//! for a specific method-operation of call-chain and thus shows how this value evolves over time.
//! In the current version the output goes to a CSV-file that can be read in Excel. In this version the data-transformation and the output generation are coupled. In a next version we should
//! split these phases such that we a separation of concerns and open new options to use the data.

mod anomalies;
mod call_chain_reporter;
mod dataseries;
mod key;
mod proc_oper_stats_reporter;
mod stats_rec_reporter;
mod stitch_list;
mod stitch_tables;
mod stitched;
mod stitched_line;
mod stitched_set;

pub use anomalies::AnomalyParameters;
pub use stitch_list::StitchList;
pub use stitched::{StitchParameters, Stitched};
