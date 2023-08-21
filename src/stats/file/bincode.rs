//!  Write the statistics to a JSON file and read them back in memory
//!
use super::super::StatsRec;
use std::{fs, io};

use super::StatsRecJson;

pub fn dump_file(file_name: &str, stats: StatsRec) {
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = io::BufWriter::new(f);

    let srj: StatsRecJson = stats.into();
    if let Err(err) = bincode::serialize_into(writer, &srj) {
        panic!("Dump of data to file {file_name} failed.\n\tError: {err:?}");
    };
}
