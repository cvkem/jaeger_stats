use std::collections::HashMap;
use crate::{Spans};

#[derive(Debug, Default)]
pub struct PathStats {
    pub count: usize,
    pub depth: usize,
    pub looped: Vec<String>
}

impl PathStats {
    pub fn new() -> Self {
        PathStats{count: 0, depth: 0, looped: Vec::new()}
    }
}

#[derive(Debug, Default)]
pub struct Stats {
    count: usize,
    method: HashMap<String, usize>,
    call_chain: HashMap<String, PathStats>,
}

impl Stats {
    pub fn new() -> Self {
        Stats{count: 0, method: HashMap::new(), call_chain: HashMap::new()}
    }
}


//type StatsMap = HashMap<String, Stats>;

#[derive(Debug, Default)]
pub struct StatsMap (HashMap<String, Stats>);

impl StatsMap {

    pub fn new() -> Self {
        StatsMap(HashMap::new())
    }

    pub fn extend_statistics(&mut self, spans: &Spans) {

        spans
            .iter()
            .enumerate()
            .for_each(|(idx, span)| {
                let proc = match &span.process {
                    Some(p) => p.name.to_owned(),
                    None => "-".to_owned()
                };
                let method = &span.operation_name;
                let update_stat = |stat: &mut Stats| {
                    stat.count += 1;

                    // add a count per method
                    stat.method
                        .entry(method.to_owned())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);

                    // add call-chain stats
                    println!(" get call-chain");
                    let call_chain = get_call_chain(idx, &spans);
                    println!(" get looped for {call_chain:?}");
                    let depth = call_chain.len();
                    let looped = get_duplicates(&call_chain);
                    println!(" get call_chain_str");
                    let call_chain_str = call_chain
                        .into_iter()
                        .fold(String::new(), |a, b| a + &b + " | ");
                    stat.call_chain
                        .entry(call_chain_str)
                        .and_modify(|ps| ps.count += 1)
                        .or_insert_with(|| PathStats{count: 1, depth, looped});
                    
                };
                self.0
                    .entry(proc)
                    .and_modify(update_stat)            
                    .or_insert_with(|| {
                        let mut stat = Stats::new();
                        update_stat(&mut stat);
                        stat
                    });
            });
//        self  // return self for chaining
    }
    
    pub fn to_csv_string(&self) -> String {
        let mut s = Vec::new();

        let mut data: Vec<_> = self.0.iter().collect();
        data.sort_by(|a,b| { a.0.cmp(&b.0)});

        s.push("process; count".to_owned());
        data.iter()
            .for_each(|(k, stat)| {
                let line = format!("{k};{}", stat.count);
                s.push(line);
            });
        s.push("\n".to_owned());

        s.push("process-method; count".to_owned());
        data.iter()
            .for_each(|(k, stat)| {
                stat.method
                    .iter()
                    .for_each(|(method, count)| {
                        let line = format!("{k}/{method};{count}");
                        s.push(line);
                            })
            });
            s.push("\n".to_owned());

            s.push("process; call_chain; depth; count; looped; revisit".to_owned());
            data.iter()
                .for_each(|(k, stat)| {
                    stat.call_chain
                        .iter()
                        .for_each(|(cc_key, path_stats)| {
                            let line = format!("{k}; {cc_key}; {}; {}; {}; {:?}", 
                                path_stats.depth, path_stats.count, path_stats.looped.len()> 0, path_stats.looped);
                            s.push(line);
                                })
                });
                s.push("\n".to_owned());
    
            s.join("\n")
    }
}


/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn basic_stats(spans: &Spans) -> HashMap<String, u32> {
    let mut stats = HashMap::new();
    spans
        .iter()
        .for_each(|span| {
            let proc = match &span.process {
                Some(p) => &p.name[..],
                None => "-"
            };
            let proc_method = format!("{}/{}", proc, span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}


fn get_parent_processes(parent_idx: Option<usize>, spans: &Spans) -> String {
    let mut parents = Vec::new();
    let mut par_idx = parent_idx;
    loop {
        match par_idx {
            None => break,
            Some(idx) => {
                let par_span = &spans[idx];
                parents.push(match &par_span.process {
                    Some(p) => p.name.to_owned(),
                    None => "-".to_owned()
                });
                par_idx = par_span.parent;
            }
        }
    }
    let res = parents
        .into_iter()
        .rev()
        .fold(String::new(), |a, b| a + &b + "|");
    res
}


/// parent_call_chain returns a Vec<String> that shows the full call_chain from top to bottom
fn get_call_chain(idx: usize, spans: &Spans) -> Vec<String> {
    let span = &spans[idx];
    // find the root and allocate vector
    let mut call_chain = match span.parent {
         None =>  Vec::new(),
         Some(idx) => get_call_chain(idx, spans)
        };
    // and push all proces names starting from the root
    call_chain.push(match &span.process {
        Some(p) => p.name.to_owned(),
        None => "-".to_owned()
    });
    call_chain
}

/// get all values that appear more than once in the list of strings, while being none-adjacent.
fn get_duplicates(names: &Vec<String>) -> Vec<String> {
    let mut duplicates = Vec::new();
    for idx in 0..names.len() {
        let nme = &names[idx];
        let mut j = 0;
        loop {
            if j >= duplicates.len() {
                break;
            }
            if duplicates[j] == *nme {
                break;
            }
            j += 1;
        }
        if j < duplicates.len() {
            continue;
        }
        //  nme does not exist in duplicates yet, so find it in names
        let mut j = idx + 2; // Step by 2 as we want to prevent matching sub-sequent GET calls
        loop {
            if j>= names.len() || names[j] == *nme {
                break;
            }
            j += 1;
        }
        if j < names.len() {
            duplicates.push(nme.to_owned());
        }
    }
    duplicates
}

/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn chained_stats(spans: &Spans) -> HashMap<String, u32> {
    let mut stats = HashMap::new();
    spans
        .iter()
        .enumerate()
        .for_each(|(idx, span)| {
            let proc = match &span.process {
                Some(p) => &p.name[..],
                None => "-"
            };
            let parents_str = get_call_chain(idx, spans)
                .into_iter()
                .fold(String::new(), |a, b| a + &b + "|");
            let proc_method = format!("{parents_str}{proc}/{}", span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}

