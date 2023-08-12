use super::stitch_tables::{append_basic_stats, append_callchain_table, append_method_table};
use crate::{
    aux::{read_lines, write_string_to_file},
    stats::StatsRec,
};
use std::{error::Error, ffi::OsString, path::Path};

#[derive(Debug)]
pub struct StitchSourceItem{
    pub column: Option<u32>,
    pub description: String
}

impl StitchSourceItem {

    pub fn new(column: Option<u32>, description: &str) -> Self {
        Self{column, description: description.to_string()}
    }

    pub fn to_csv_string(&self) -> String {
        match self.column {
            Some(col) => format!("{}; {}", col, self.description),
            None => format!(" ; {}", self.description)
        }
    }
}

#[derive(Default,Debug)]
pub struct StitchSources( Vec<StitchSourceItem> );


impl StitchSources {

    /// add an unnumbered line (comment line)
    fn add_unnumbered(&mut self, description: &str) {
        self.0.push(StitchSourceItem::new(None, description));
    }

            /// add an numbered line, where the number is the sequence-number for this path (so add path first)
    fn add_numbered(&mut self, description: &str) {
        let column = self.0.iter().filter(|x| x.column.is_some()).count() as u32;
        self.0.push(StitchSourceItem::new(Some(column), description));
    }

    fn csv_output(&self ) -> Vec<String> {
        self.0.iter().map(|line| line.to_csv_string()).collect()
    }
}


#[derive(Default, Debug)]
pub struct StitchList {
    pub lines: StitchSources,           // numbered Lines including comments
    pub paths: Vec<Option<OsString>>, // a None represents a slot that is not filled (will become an empty column)
}

impl StitchList {

    pub fn new() -> Self {
        Default::default()
    }

    fn add_path(&mut self, base_path: &Path, path: Option<&str>) {
        match path {
            Some(path) => {
                // skip comments at the tail of the path-string
                let mut path = match path.find('#') {
                    Some(pos) => path[0..pos].trim(),
                    None => path,
                };
                // correct base-path for ".." on path
                let mut base_path = base_path.to_path_buf();
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

    /// Reading all data of a stitchlist in a Vector.
    /// This is still columnar data, with one data-set per analysis. Each column is an Option<StatsRec> as missing columns are supported during stitching.
    /// Missing columns are needed to mimick the actual timeline (and thus gaps in the timeline) for a correct linear regression analysis.
    pub fn read_data(&self) -> Vec<Option<StatsRec>> {
        self
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
            .collect()
    }

    /// Read all stitched data and write it out to a CSV files
    /// TODO: refactor to separate the CSV-output phase from the actual transposition and structuring of the data.
    pub fn write_stitched_csv(self, path: &Path) {
        let data = self.read_data();

        let mut csv_string = Vec::new();
        csv_string.push("Contents:".to_owned());
        (0..6).for_each(|_| csv_string.push(String::new()));

        csv_string[1] = format!(
            "\trow {}: Column_numbering (based on the input-files",
            csv_string.len()
        );
        csv_string.append(&mut self.lines.csv_output());

        csv_string[2] = format!("\trow {}: Basic statistics", csv_string.len());
        append_basic_stats(&mut csv_string, &data);
        csv_string[3] = format!(
            "\trow {}: Statistics per BSP/operation combination",
            csv_string.len()
        );
        append_method_table(&mut csv_string, &data);
        csv_string[4] = format!("\trow {}: Statistics per call-chain (path from the external end-point to the actual BSP/operation (detailled information)", csv_string.len());
        append_callchain_table(&mut csv_string, &data);

        match write_string_to_file(path.to_str().unwrap(), csv_string.join("\n")) {
            Ok(()) => (),
            Err(err) => println!(
                "Writing file '{}' failed with Error: {err:?}",
                path.display()
            ),
        };
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
            if !l.is_empty() {
                let ch = l.chars().next().unwrap();
                match ch {
                    '#' => sl.lines.add_unnumbered(l),
                    '%' => {
                        sl.add_path(&base_path, None);
                        sl.lines.add_numbered(l);
                    }
                    _ => {
                        sl.add_path(&base_path, Some(l));
                        sl.lines.add_numbered(l);
                    }
                }
            }
            sl
        }))
    }

}

