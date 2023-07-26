use std::{
    collections::HashSet,
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf}};
use crate::{
    aux::{read_lines, write_string_to_file},
    stats_json::StatsRecJson};



#[derive(Default, Debug)]
pub struct StitchList {
    pub lines: Vec<String>,  // numbered Lines including comments
    pub paths: Vec<Option<OsString>>  // a None represents a slot that is not filled (will become an empty column)
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
                    None => path            
                };
                // correct base-path for ".." on path
                let mut base_path = base_path.clone();
                while path.starts_with("../") || path.starts_with(r"..\") {
                    path = &path[3..];
                    println!("TMP skipped .. now using {path}");
                    if !base_path.pop() {
                        println!("can not backtrack via .. beyond the root basepath");
                    }
                }
                        
                base_path.push(Path::new(path));
                println!("base_path now is {base_path:?}");
                base_path.canonicalize().
                    map_err(|err| {eprintln!("\nFailed to handle path {base_path:?}. File probably does not exist!!"); err}).unwrap();
                self.paths.push(Some(base_path.into_os_string()));        
            },
            None => self.paths.push(None)
        }
    }

    /// add an numbered line, where the number is the sequence-number for this path (so add path first)
    fn add_numbered(&mut self, l: &str) {
        let line = format!("{}; {}", self.paths.len(), l);
        self.lines.push(line);
    }

    pub fn write_stitched_csv(self, path: &Path) {
        let data = self.paths
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                if let Some(p) = p {
                    println!("{}: Reading {p:?}", idx+1);
                    Some(StatsRecJson::read_file(p).expect("Failed to read JSON-file"))
                } else {
                    println!("{}: No Data", idx+1);
                    None
                }
            })
            .collect();
        
        let mut csv_string = self.lines;

        csv_string.push("\n\n# Method table".to_owned());
        append_method_table(&mut csv_string, &data);

        csv_string.push("\n\n# Call-chain table".to_owned());
        append_callchain_table(&mut csv_string, &data);

        match write_string_to_file(path.to_str().unwrap(), csv_string.join("\n")) {
            Ok(()) => (),
            Err(err) => println!("Writing file '{}' failed with Error: {err:?}", path.display())
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

    Ok(read_lines(path)?
        .fold(StitchList::new(), |mut sl, l| {
            let l = l.unwrap();
            let l = l.trim();
            if l.len() > 0 {
                let ch = l.chars().next().unwrap();
                match ch {
                    '#' => sl.add_unnumbered(l),
                    '%' => {
                        sl.add_path(&base_path, None);
                        sl.add_numbered(l);
                    },
                    _  => {
                       sl.add_path(&base_path, Some(l));
                       sl.add_numbered(l);
                    }    
                }
            }
            sl
        }))
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Key {
    process: String,
    operation: String
}

// struct MethodStatsReporter{
//     buffer: &mut Vec<String>,
//     data: &Vec<Option<StatsRecJson>>,
//     keys: 
// }

// impl MethodStatsReporter {

//     fn build_reporter(buffer: &mut Vec<String>, data: &Vec<Option<StatsRecJson>>) -> Self {
//         Self{buffer, data, keys}
//     }

// }

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
fn append_method_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRecJson>>) {

    // find a deduplicated set of all keys
    let mut keys  = HashSet::new();
    data.iter()
        .for_each(|str| {
            if let Some(str) = str {
                str.stats.iter()
                    .for_each(|(proc_key, st)| {
                        st.method.0.iter()
                            .for_each(|(oper_key, _)| _ = keys.insert(Key{process: proc_key.to_owned(), operation: oper_key.to_owned()}))
                    })
            }
        });
    let mut keys: Vec<_> = keys.into_iter().collect();
    keys.sort();

    keys.into_iter()
        .for_each(|Key{process, operation}| {
            let meth_stats: Vec<_> = data.iter()
                .map(|str| {
                    match str {
                        Some(str) => {
                            match str.stats.get(&process) {
                                Some(st) => match st.method.0.get(&operation) {
                                    Some(oper) => Some(oper),
                                    None => None
                                },
                                None => None
                            }
                        }
                        None => None
                    }
                })
                .collect();
            // min
            let values = meth_stats.iter()
                .map(|ms| ms.map_or("".to_owned(),|x |x.get_min_millis_str()))
                .collect::<Vec<_>>()
                .join("; ");
            buffer.push(format!("{process}/{operation}; min_millis; {values}"));
            // average
            let values = meth_stats.iter()
                .map(|ms| ms.map_or("".to_owned(),|x |x.get_avg_millis_str()))
                .collect::<Vec<_>>()
                .join("; ");
            buffer.push(format!("{process}/{operation}; avg_millis; {values}"));
            // max
            let values = meth_stats.iter()
                .map(|ms| ms.map_or("".to_owned(),|x |x.get_max_millis_str()))
                .collect::<Vec<_>>()
                .join("; ");
            buffer.push(format!("{process}/{operation}; max_millis; {values}"));

            // println!(" {process}/{operation}")

        });

}



/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
fn append_callchain_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRecJson>>) {


}
