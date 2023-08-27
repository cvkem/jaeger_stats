//!  Write the statistics to a JSON file and read them back in memory
//!
mod bincode;
mod json;
mod operation_stats_json;

use super::StatsRec;

pub use operation_stats_json::{OperationStatsJson, StatsRecJson};

pub fn write_stats(file_name: &str, stats: StatsRec, ext: &str) {
    let file_name = file_name.replace(".csv", &format!(".{ext}"));
    match ext {
        "json" => json::dump_file(&file_name, stats),
        "bincode" => bincode::dump_file(&file_name, stats),
        unknown => panic!("Unknown output format: '{unknown}'"),
    }
}
