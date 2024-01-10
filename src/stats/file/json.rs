//!  Write the statistics to a JSON file and read them back in memory
//!
use super::super::StatsRec;
use std::{fs, io};

use super::StatsRecJson;

/// The dump of the file consumes the StatsRec data, as it needs to be turned into a StatsRecJson.
/// TODO: consider whether it is possible to make a StartRecJson based on References (to allow for an efficient write-operation without consuming the input StatsRec)
pub fn dump_file(file_name: &str, stats: StatsRec) {
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = io::BufWriter::new(f);
    let srj: StatsRecJson = stats.into();
    // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human readible)
    match serde_json::to_writer_pretty(writer, &srj) {
        Ok(()) => (),
        Err(err) => panic!("failed to Serialize !! {err:?}"),
    }
}
