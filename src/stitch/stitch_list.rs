use super::{
    call_chain_reporter::{CCReportItem, CallChainReporter},
    key::Key,
    method_stats_reporter::{MSReportItem, MethodStatsReporter},
    stats_rec_reporter::{SRJReportItem, StatsRecReporter},
};
use crate::{
    aux::{read_lines, write_string_to_file},
    stats::StatsRec,
};
use std::{
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Default, Debug)]
pub struct StitchList {
    pub lines: Vec<String>,           // numbered Lines including comments
    pub paths: Vec<Option<OsString>>, // a None represents a slot that is not filled (will become an empty column)
}

impl StitchList {
    pub fn new() -> Self {
        Default::default()
    }

    /// add an unnumbered line (comment line)
    fn add_unnumbered(&mut self, l: &str) {
        self.lines.push(format!(";{l}"));
    }

    fn add_path(&mut self, base_path: &PathBuf, path: Option<&str>) {
        match path {
            Some(path) => {
                // skip comments at the tail of the path-string
                let mut path = match path.find('#') {
                    Some(pos) => &path[0..pos].trim(),
                    None => path,
                };
                // correct base-path for ".." on path
                let mut base_path = base_path.clone();
                while path.starts_with("../") || path.starts_with(r"..\") {
                    path = &path[3..];
                    if !base_path.pop() {
                        panic!("can not backtrack via .. beyond the root basepath {base_path:?} for path {path}");
                    }
                }

                base_path.push(Path::new(path));
                println!("base_path now is {base_path:?}");
                base_path
                    .canonicalize()
                    .map_err(|err| {
                        eprintln!(
                            "\nFailed to handle path {base_path:?}. File probably does not exist!!"
                        );
                        err
                    })
                    .unwrap();
                self.paths.push(Some(base_path.into_os_string()));
            }
            None => self.paths.push(None),
        }
    }

    /// add an numbered line, where the number is the sequence-number for this path (so add path first)
    fn add_numbered(&mut self, l: &str) {
        let line = format!("{}; {}", self.paths.len(), l);
        self.lines.push(line);
    }

    pub fn write_stitched_csv(self, path: &Path) {
        let data = self
            .paths
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                if let Some(p) = p {
                    println!("{}: Reading {p:?}", idx + 1);
                    Some(StatsRec::read_file(p).expect("Failed to read JSON-file"))
                } else {
                    println!("{}: No Data", idx + 1);
                    None
                }
            })
            .collect();

        let mut csv_string = self.lines;

        append_basic_stats(&mut csv_string, &data);
        append_method_table(&mut csv_string, &data);
        append_callchain_table(&mut csv_string, &data);

        match write_string_to_file(path.to_str().unwrap(), csv_string.join("\n")) {
            Ok(()) => (),
            Err(err) => println!(
                "Writing file '{}' failed with Error: {err:?}",
                path.display()
            ),
        };
    }
}

/// Read a stitch-list file and return a struct showing the contents.
pub fn read_stitch_list(path: &Path) -> Result<StitchList, Box<dyn Error>> {
    let base_path = path
        .canonicalize()
        .expect("Failed to make canonical stitch-list-path. Path probably does not exist!")
        .parent()
        .expect("Could not extract base_path of stitch-list")
        .to_path_buf();

    Ok(read_lines(path)?.fold(StitchList::new(), |mut sl, l| {
        let l = l.unwrap();
        let l = l.trim();
        if l.len() > 0 {
            let ch = l.chars().next().unwrap();
            match ch {
                '#' => sl.add_unnumbered(l),
                '%' => {
                    sl.add_path(&base_path, None);
                    sl.add_numbered(l);
                }
                _ => {
                    sl.add_path(&base_path, Some(l));
                    sl.add_numbered(l);
                }
            }
        }
        sl
    }))
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
fn append_basic_stats(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("\n\n# Basic statistics over alle stitched files".to_owned());
    let mut report_items = Vec::new();
    report_items.push(SRJReportItem::new("num_files", |srj| {
        srj.num_files.to_string()
    }));
    report_items.push(SRJReportItem::new("num_traces", |srj| {
        srj.trace_id.len().to_string()
    }));
    report_items.push(SRJReportItem::new("avg_duration_micros", |srj| {
        (srj.duration_micros.iter().sum::<i64>() / srj.duration_micros.len() as i64).to_string()
    }));
    report_items.push(SRJReportItem::new("avg_duration_micros", |srj| {
        (srj.time_to_respond_micros.iter().sum::<i64>() / srj.time_to_respond_micros.len() as i64)
            .to_string()
    }));

    let mut reporter = StatsRecReporter::new(buffer, data, report_items);
    reporter.append_report()
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
fn append_method_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("\n\n# Method table".to_owned());

    // build the stack of reports that need to be calculated
    let mut report_items = Vec::new();
    report_items.push(MSReportItem::new("count", |msv, _, _| {
        msv.count.to_string()
    }));
    report_items.push(MSReportItem::new(
        "Occurance percentage",
        |msv, _, num_traces| (msv.count as f64 / num_traces as f64).to_string(),
    ));
    report_items.push(MSReportItem::new("rate (avg)", |msv, num_files, _| {
        msv.get_avg_rate_str(num_files)
    }));
    //    report_items.push(MSReportItem::new("rate (median)", |msv, num_files, _| msv.get_median_rate_str(num_files)));
    report_items.push(MSReportItem::new("min_millis", |msv, _, _| {
        msv.get_min_millis_str()
    }));
    report_items.push(MSReportItem::new("avg_millis", |msv, _, _| {
        msv.get_avg_millis_str()
    }));
    report_items.push(MSReportItem::new("max_millis", |msv, _, _| {
        msv.get_max_millis_str()
    }));
    report_items.push(MSReportItem::new("frac_not_http_ok", |msv, _, _| {
        msv.get_frac_not_http_ok_str()
    }));
    report_items.push(MSReportItem::new("frac_error_logs", |msv, _, _| {
        msv.get_frac_error_log_str()
    }));

    // Build a reporter that handles shows the items defined in the report_items. Each item is a data-column.
    let mut reporter = MethodStatsReporter::new(buffer, data, report_items);

    // Find all keys and generate an output line for each of these keys.
    let keys = reporter.get_keys();
    keys.into_iter()
        .for_each(|Key { process, operation }| reporter.append_report(process, operation));
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
fn append_callchain_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("\n\n# Call-chain table".to_owned());
    // build the stack of reports that need to be calculated
    let mut report_items = Vec::new();
    report_items.push(CCReportItem::new("count", |msv, _, _| {
        msv.count.to_string()
    }));
    report_items.push(CCReportItem::new(
        "Occurance percentage",
        |msv, _, num_traces| (msv.count as f64 / num_traces as f64).to_string(),
    ));
    report_items.push(CCReportItem::new("rate (avg)", |msv, num_files, _| {
        msv.get_avg_rate_str(num_files)
    }));
    //    report_items.push(CCReportItem::new("rate (median)", |msv, num_files, _| msv.get_median_rate_str(num_files)));
    report_items.push(CCReportItem::new("min_millis", |msv, _, _| {
        msv.get_min_millis_str()
    }));
    report_items.push(CCReportItem::new("avg_millis", |msv, _, _| {
        msv.get_avg_millis_str()
    }));
    report_items.push(CCReportItem::new("max_millis", |msv, _, _| {
        msv.get_max_millis_str()
    }));

    report_items.push(CCReportItem::new("http_not_ok_count", |msv, _, _| {
        msv.get_frac_not_http_ok_str()
    }));
    report_items.push(CCReportItem::new("num_error_logs", |msv, _, _| {
        msv.get_frac_error_log_str()
    }));

    // Build a reporter that handles shows the items defined in the report_items. Each item is a data-column.
    let mut reporter = CallChainReporter::new(buffer, data, report_items);

    // Find all keys and generate an output line for each of these keys.
    let keys = reporter.get_keys();
    keys.into_iter().for_each(|k| reporter.append_report(k));
}
