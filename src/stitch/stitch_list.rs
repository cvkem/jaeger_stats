use crate::{
    stats::StatsRec,
    utils::{clean_os_string, extend_with_base_path, extract_base_path, is_rooted_path, read_lines},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, ffi::OsString, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StitchSourceItem {
    pub column: Option<u32>,
    pub description: String,
}

impl StitchSourceItem {
    pub fn new(column: Option<u32>, description: &str) -> Self {
        Self {
            column,
            description: description.to_string(),
        }
    }

    pub fn to_csv_string(&self) -> String {
        match self.column {
            Some(col) => format!("{}; {}", col, self.description),
            None => format!(" ; {}", self.description),
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct StitchSources(pub Vec<StitchSourceItem>);

impl StitchSources {
    /// add an unnumbered line (comment line)
    fn add_unnumbered(&mut self, description: &str) {
        self.0.push(StitchSourceItem::new(None, description));
    }

    /// add an numbered line, where the number is the sequence-number for this path (so add path first)
    fn add_numbered(&mut self, description: &str) {
        let column = self.0.iter().filter(|x| x.column.is_some()).count() as u32;
        self.0
            .push(StitchSourceItem::new(Some(column), description));
    }

    pub fn csv_output(&self) -> Vec<String> {
        self.0.iter().map(|line| line.to_csv_string()).collect()
    }

    //TODO: This list already should contain the correct list of labels, as extracted from the description based on a provided pattern.
}

#[derive(Default, Debug)]
pub struct StitchList {
    pub lines: StitchSources,         // numbered Lines including comments
    pub paths: Vec<Option<OsString>>, // a None represents a slot that is not filled (will become an empty column)
}

impl StitchList {
    pub fn new() -> Self {
        Default::default()
    }

    fn add_path(&mut self, base_path: &Path, path: Option<&str>) {
        let path = match path {
            Some(path) => Some(if is_rooted_path(path) { clean_os_string(path) } else {extend_with_base_path(base_path, path)}),
            None => None,
        };
        self.paths.push(path);
    }

    /// Reading all data of a stitchlist in a Vector.
    /// This is still columnar data, with one data-set per analysis. Each column is an Option<StatsRec> as missing columns are supported during stitching.
    /// Missing columns are needed to mimick the actual timeline (and thus gaps in the timeline) for a correct linear regression analysis.
    pub fn read_data(&self) -> Vec<Option<StatsRec>> {
        self.paths
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                if let Some(p) = p {
                    println!("{}: Reading file '{p:?}'", idx + 1);
                    Some(StatsRec::read_file(p).expect("Failed to read JSON-file"))
                } else {
                    println!("{}: No Data", idx + 1);
                    None
                }
            })
            .collect()
    }

    /// Read a stitch-list file and return a struct showing the contents.
    pub fn read_stitch_list(path: &Path) -> Result<StitchList, Box<dyn Error>> {
        let base_path = extract_base_path(path);

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
