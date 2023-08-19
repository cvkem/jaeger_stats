//!  Write the statistics to a JSON file and read them back in memory
//!
use super::super::StatsRec;
use std::{fs, io};

use super::StatsRecJson;

pub fn dump_file(file_name: &str, stats: StatsRec) {
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = io::BufWriter::new(f);
    let srj: StatsRecJson = stats.into();
    // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human readible)
    match serde_json::to_writer_pretty(writer, &srj) {
        Ok(()) => (),
        Err(err) => panic!("failled to Serialize !! {err:?}"),
    }
}
