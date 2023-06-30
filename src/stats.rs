use std::{collections::HashMap};
use crate::{
    Trace,
    span::Spans};
    use chrono::{
        DateTime,
        Utc};


#[derive(Debug, Default)]
pub struct PathStats {
    pub method: String,
    pub count: usize,
    pub depth: usize,
    pub duration_micros: Vec<u64>,
    pub is_leaf: bool,
    pub looped: Vec<String>
}

impl PathStats {
    pub fn new() -> Self {
        Default::default()
    }

    fn report_stats_line(&self, key: &str, cc_key: &str, n: f64) -> String {
        let min_millis = *self.duration_micros.iter().min().expect("Not an integer") as f64 / 1000 as f64;
        let avg_millis = self.duration_micros.iter().sum::<u64>() as f64 / (1000 as f64 * self.duration_micros.len() as f64);
        let max_millis = *self.duration_micros.iter().max().expect("Not an integer") as f64 / 1000 as f64;
        let method = &self.method;
        let line = format!("{key}/{method}; {}; {}; {}; {}; {:?}; {}; {}; {}; {}", 
            self.is_leaf, self.depth, self.count, self.looped.len()> 0, 
            self.looped, cc_key, 
            format_float(min_millis), format_float(avg_millis), format_float(max_millis));
        line
    }

}

#[derive(Debug, Default)]
pub struct Stats {
    num_received_calls: usize,  // inbound calls to this process
    num_outbound_calls: usize,  // outbound calls to other processes
    method: HashMap<String, usize>,
    method_cache_suffix: HashMap<String, usize>,  // methods in a cache-chain have a suffix.
    call_chain: HashMap<String, PathStats>,
}

impl Stats {
    pub fn new() -> Self {
        Default::default()
    }
}


#[derive(Debug, Default)]
pub struct StatsMap {
    pub trace_id: Vec<String>,
    pub root_call: Vec<String>,
    pub num_spans: Vec<usize>,
    pub start_dt: Vec<DateTime<Utc>>,
    pub end_dt: Vec<DateTime<Utc>>,
    pub duration_micros: Vec<u64>,
    pub time_to_respond_micros: Vec<u64>,
    pub caching_process: Vec<String>,
    stats: HashMap<String, Stats>
}

impl StatsMap {

    pub fn new(caching_process: Vec<String>) -> Self {
        StatsMap{
            caching_process,
            ..Default::default()}
    }


    // /// get_cache_suffix determines whether caching processes (services) are in the call-chain and if so returns a suffix to represent it.
    // ///  Could not be used, using an independent function with the same name instead.
    // pub fn get_cache_suffix(&self, call_chain: &Vec<(String, String)>) -> String {
    //     if self.caching_process.len() == 0 {
    //         return "".to_owned()
    //     }
    //     let mut cached = Vec::new();

    //     call_chain.iter()
    //         .for_each(|(proc, method)| {
    //             // method does not matter. These things can be cached too.
    //             // match &method[..] {
    //             //     "GET" | "POST" | "HEAD" | "QUERY" => (),  // ignore these methods
    //             //     _ => 
    //                 match self.caching_process.iter().find(|&s| *s == *proc) {
    //                     Some(_) => {
    //                         cached.push(proc.to_owned())},
    //                     None => ()
    //                 }
    //             //}
    //         });
    //     format!(" [{}]", cached.join(", "))
    // }


    pub fn extend_statistics(&mut self, trace: &Trace) {

        let spans = &trace.spans;

        self.trace_id.push(trace.trace_id.to_owned());
        self.root_call.push(trace.root_call.to_owned());
        self.num_spans.push(trace.spans.len());
        self.start_dt.push(trace.start_dt);
        self.end_dt.push(trace.end_dt);
        self.duration_micros.push(trace.duration_micros);
        self.time_to_respond_micros.push(trace.time_to_respond_micros);

        spans
            .iter()
            .enumerate()
            .for_each(|(idx, span)| {
                let proc = span.get_process_str().to_owned();
                let method = &span.operation_name;
                let update_stat = |stat: &mut Stats| {
                    match &method[..] {
                        "GET" | "POST" | "HEAD" | "QUERY" => stat.num_outbound_calls += 1,
                        _ => stat.num_received_calls += 1 
                    }

                    // add a count per method
                    stat.method
                        .entry(method.to_owned())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);

                    // add a count per method_including-cached
                    let call_chain = get_call_chain(idx, &spans);
                    {
                        // next line fails as we use self with a self method and closure
                        let cache_suffix = get_cache_suffix(&self.caching_process, &call_chain);
                        let method_cached = method.to_owned() + &cache_suffix;
                        stat.method_cache_suffix
                            .entry(method_cached.to_owned())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    }

                    // add call-chain stats
                    let depth = call_chain.len();
                    let looped = get_duplicates(&call_chain);
                    let duration_micros = span.duration_micros;
                    let call_chain_str = call_chain
                        .into_iter()
                        .fold(String::new(), |a, b| a + &b.0 + "/" + &b.1 + " | ");
                    stat.call_chain
                        .entry(call_chain_str)
                        .and_modify(|ps| {
                            ps.count += 1;
                            ps.duration_micros.push(duration_micros);})
                        .or_insert_with(|| {
                            let dms: Box<[_]> = Box::new([duration_micros]);
                            let duration_micros = dms.into_vec();
                            let method = method.to_owned();
                            PathStats{method, count: 1, depth, duration_micros, is_leaf: span.is_leaf, looped}
                        });
                    
                };

                self.stats
                    .entry(proc)
                    .and_modify(update_stat)            
                    .or_insert_with(|| {
                        let mut stat = Stats::new();
                        update_stat(&mut stat);
                        stat
                    });
            });
    }


    pub fn to_csv_string(&self) -> String {
        let mut s = Vec::new();
        let num_traces = self.trace_id.len() as u64;

        match num_traces {
            0 => panic!("No data in Stats"),
            1 => {
                s.push(format!("trace_id:; {}", self.trace_id[0]));
                s.push(format!("root_call:; {}", self.root_call[0]));
                s.push(format!("num_spans:; {}", self.num_spans[0]));
                s.push(format!("start_dt; {:?}", self.start_dt[0]));
                s.push(format!("end_dt:; {:?}", self.end_dt[0]));
                s.push(format!("duration_micros:; {}", self.duration_micros[0]));
                s.push(format!("time_to_respond_micros:; {}", self.time_to_respond_micros[0]));
        
            },
            n => {
                s.push(format!("trace_ids:; {:?}", self.trace_id));
                s.push(format!("num_traces:; {num_traces}"));
                s.push(format!("MIN(num_spans):; {:?}", self.num_spans.iter().min().unwrap()));
                s.push(format!("AVG(num_spans):; {:?}", self.num_spans.iter().sum::<usize>() as u64/n));
                s.push(format!("MAX(num_spans):; {:?}", self.num_spans.iter().max().unwrap()));
                s.push(format!("root_call_stats:; {}", root_call_stats(&self.root_call)));
                s.push(format!("root_calls:; {:?}", root_call_list(&self.trace_id, &self.root_call)));
                s.push(format!("start_dt; {:?}", self.start_dt));
                s.push(format!("end_dt:; {:?}", self.end_dt));
                s.push(format!("MIN(duration_micros):; {:?}", self.duration_micros.iter().min().unwrap()));
                s.push(format!("AVG(duration_micros):; {:?}", self.duration_micros.iter().sum::<u64>()/n));
                s.push(format!("MAX(duration_micros):; {:?}", self.duration_micros.iter().max().unwrap()));
                s.push(format!("duration_micros:; {:?}", self.duration_micros));
                s.push(format!("MIN(time_to_respond_micros):; {:?}", self.time_to_respond_micros.iter().min().unwrap()));
                s.push(format!("AVG(time_to_respond):; {:?}", self.time_to_respond_micros.iter().sum::<u64>()/n));
                s.push(format!("MAX(time_to_respond_micros):; {:?}", self.time_to_respond_micros.iter().max().unwrap()));
                s.push(format!("time_to_respond_micros:; {:?}", self.time_to_respond_micros));        
            }
        }
        s.push("\n".to_owned());

        let mut data: Vec<_> = self.stats.iter().collect();
        data.sort_by(|a,b| { a.0.cmp(&b.0)});

        s.push("Process; Num_received_calls; Num_outbound_calls".to_owned());
        data.iter()
            .for_each(|(k, stat)| {
                let freq_rc = stat.num_received_calls as f64/ num_traces as f64;
                let freq_oc = stat.num_outbound_calls as f64/ num_traces as f64;
                let line = format!("{k}; {}; {}; {}; {}", 
                    stat.num_received_calls, stat.num_outbound_calls,
                    format_float(freq_rc), format_float(freq_oc));
                s.push(line);
            });
        s.push("\n".to_owned());

        s.push("Process-method; Count; Freq".to_owned());
        data.iter()
            .for_each(|(k, stat)| {
                stat.method
                    .iter()
                    .for_each(|(method, count)| {
                        let freq = *count as f64/ num_traces as f64;
                        let line = format!("{k}/{method}; {count}; {}", format_float(freq));
                        s.push(line);
                            })
            });
        s.push("\n".to_owned());

        s.push("Process-method-Cached; Count; Freq".to_owned());
        data.iter()
            .for_each(|(k, stat)| {
                stat.method_cache_suffix
                    .iter()
                    .for_each(|(method_cached, count)| {
                        let freq = *count as f64/ num_traces as f64;
                        let line = format!("{k}/{method_cached}; {count}; {}", format_float(freq));
                        s.push(line);
                            })
            });
        s.push("\n".to_owned());


        s.push("Process; Is_leaf; Depth; Count; Looped; Revisit; Call_chain; min_millis; avg_millis; max_millis".to_owned());
        let num_traces = num_traces as f64; 
        data.iter()
            .for_each(|(k, stat)| {
                stat.call_chain
                    .iter()
                    .for_each(|(cc_key, path_stats)| s.push(path_stats.report_stats_line(k, cc_key, num_traces)))
            });
            s.push("\n".to_owned());
    
            s.join("\n")
    }
}



/// get_cache_suffix determines whether cached processes are in the call-chain and if so returns a suffix to represent it.
pub fn get_cache_suffix(caching_process: &Vec<String>, call_chain: &Vec<(String, String)>) -> String {
    if caching_process.len() == 0 {
        return "".to_owned()
    }
    let mut cached = Vec::new();

    call_chain.iter()
    .for_each(|(proc, method)| {
        match &method[..] {
            "GET" | "POST" | "HEAD" | "QUERY" => (),  // ignore these methods as the inbound call has been matched already. (prevent duplicates of cached names)
            _ => match caching_process.iter().find(|&s| *s == *proc) {
                Some(_) => {
                    cached.push(proc.to_owned())},
                None => ()
            }
        }
    });
    if cached.len() > 0 {
        format!(" [{}]", cached.join(", "))
    } else {
        "".to_owned()
    }
}


/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn basic_stats(trace: &Trace) -> HashMap<String, u32> {
    let spans = &trace.spans;

    let mut stats = HashMap::new();
    spans
        .iter()
        .for_each(|span| {
            let proc = span.get_process_str();
            let proc_method = format!("{}/{}", proc, span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}


const NL_FORMAT: bool = true;


/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
fn format_float(val: f64) -> String {
    let s = format!("{}", val);
    if NL_FORMAT {
        s.replace('.',",")
    } else {
        s
    }
}


/// parent_call_chain returns the full call_chain from top to bottom showing process and called method
fn get_call_chain(idx: usize, spans: &Spans) -> Vec<(String, String)> {
    let span = &spans[idx];
    // find the root and allocate vector
    let mut call_chain = match span.parent {
         None =>  Vec::new(),
         Some(idx) => get_call_chain(idx, spans)
        };
    // and push all proces names starting from the root
    let process = span.get_process_str().to_owned();
    let method = span.operation_name.to_owned();
    call_chain.push( (process, method) );
    call_chain
}


/// get all values that appear more than once in the list of strings, while being none-adjacent.
fn get_duplicates(names: &Vec<(String, String)>) -> Vec<String> {
    let mut duplicates = Vec::new();
    for idx in 0..names.len() {
        let proc = &names[idx].0;
        let mut j = 0;
        loop {
            if j >= duplicates.len() {
                break;
            }
            if duplicates[j] == *proc {
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
            if j>= names.len() || names[j].0 == *proc {
                break;
            }
            j += 1;
        }
        if j < names.len() {
            duplicates.push(proc.to_owned());
        }
    }
    duplicates
}


/// Compute basic call statistics, which only looks at functions/operations and does not include the call path
pub fn chained_stats(trace: &Trace) -> HashMap<String, u32> {
    let spans = &trace.spans;

    let mut stats = HashMap::new();
    spans
        .iter()
        .enumerate()
        .for_each(|(idx, span)| {
            let proc = span.get_process_str();
            let parents_str = get_call_chain(idx, spans)
                .into_iter()
                .fold(String::new(), |a, b| a + &b.0 + &b.1 +" | ");
            let proc_method = format!("{parents_str}{proc}/{}", span.operation_name);
            stats.entry(proc_method).and_modify(|counter| *counter += 1).or_insert(1);
        });
    stats
}


/// root_call_stats return a list of root_calls and their count.
fn root_call_stats(root_calls: &Vec<String>) -> String {
    let mut stats = HashMap::new();
    root_calls
        .iter()
        .for_each(|call| {
            stats
                .entry(call)
                .and_modify(|counter: &mut i32| *counter += 1)
                .or_insert(1);
        });
    let mut data: Vec<_> = stats.iter().collect();
    data.sort_by(|a,b| { b.1.cmp(&a.1)});
    format!("{data:?}")
}

fn root_call_list(trace_ids: &Vec<String>, root_calls: &Vec<String>) -> String {
    let labelled: Vec<_> = trace_ids
        .iter()
        .zip(root_calls.iter())
        .map(|(tr, rc)| format!("{tr} -> {rc}"))
        .collect();
    labelled
        .join(",   ")
}