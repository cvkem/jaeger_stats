use std::iter;
use crate::{
    JaegerTrace,
    raw_jaeger::{
        JaegerSpan,
        JaegerTags},
    process_map::{build_process_map, ProcessMap, Process},
    micros_to_datetime};
use chrono::{
    DateTime,
    Utc};


#[derive(Debug, Default)]
pub struct Span{
// Process should be a reference, but that complicates things:
//    - default is not possible
//    - how to build a Vec (no copy)
//    - passing lifetimes
//pub struct Span<'a> {
    pub parent: Option<usize>,
    pub is_leaf: bool,
    pub span_id: String,
    pub operation_name: String,
    pub start_dt: DateTime<Utc>,
    pub duration_micros: u64,
//    pub process: &'a Process,
    pub process: Option<Process>,
    // optional parameters from tags
    // to see statistics on all tags run:
    //      cargo run --example collect_span_tags
    pub http_status_code: Option<i32>,
    pub db_instance: Option<String>,
    pub db_type: Option<String>,
    pub db_statement: Option<String>,
    pub warnings: Option<Vec<String>>,
    pub eb_contract: Option<String>,     // either tag 'identity.eb_contract_id'  or 'eb_contract'
}

pub type Spans = Vec<Span>;

impl Span {
    fn new(js: &JaegerSpan, proc_map: &ProcessMap) -> Self {
        let parent = None;
        let span_id = js.spanID.to_owned();
        let operation_name = js.operationName.to_owned();
        let start_dt = micros_to_datetime(js.startTime);
        let duration_micros = js.duration;
        let process = proc_map.get(&js.processID).map(|proc| proc.to_owned());
        let mut span = Span { parent, span_id, operation_name, start_dt, duration_micros, process,
            ..Default::default()};
        span.add_tags(&js.tags);
        span
        }


    fn add_tags(&mut self, tags: &JaegerTags) {
        tags
            .iter()
            .for_each(|tag| {
                match &tag.key[..] {
                    "http.status_code" => self.http_status_code = Some(tag.get_i32()), 
                    "db.instance" => self.db_instance = Some(tag.get_string()), 
                    "db.type" => self.db_instance = Some(tag.get_string()), 
                    "db.statement" => self.db_statement = Some(tag.get_string()), 
                    "identity.eb_contract_id " | "eb_contract" => self.eb_contract = Some(tag.get_string()),
                    _ => ()
                }
            })
    }

    //. get_process_name returns the string-slice of the process of this span (without the operation (method) that is called)
    pub fn get_process_str(&self) -> & str {
        match &self.process {
            Some(p) => &p.name[..],
            None => "-"
        }
    }

    // //. get_process_name returns the name of the process of this span (without the operation (method) that is called)
    // pub fn get_process_name(&self) -> String {
    //     self.get_process_str().to_owned()
    // }

}


/// mark_leafs sets the is_leaf value of each span.
fn mark_leafs(spans: &mut Spans) {
    let mut is_leaf = Vec::with_capacity(spans.len());
    (0..spans.len()).for_each(|_| is_leaf.push(true));
    spans.iter().for_each(|span| {
        match span.parent {
            None => (),
            Some(par) => is_leaf[par] = true
        }
    });

    iter::zip(spans, is_leaf).for_each(|(mut span, is_leaf)| span.is_leaf = is_leaf);
}

/// add_parents adds parent-links to spans based on the information in Vec<JaegerSpan>
fn add_parents(spans: &mut Vec<Span>, jspans: &Vec<JaegerSpan>) {
    let mut iter = iter::zip(spans, jspans);

    iter.for_each(|(mut span, jspan)| {
        match jspan.references.len() {
            0 => (), // this is the root
            1 => {
                let parentID = &jspan.references[0].spanID;
                for (idx, js) in jspans.iter().enumerate() {
                    if js.spanID[..] == parentID[..] {
                        span.parent = Some(idx);
                        break;
                    }
                }
            },
            num => panic!("Span '{}' has {num} parent-references.", jspan.spanID)
        }
    })

} 

/// build the ist of spans (including parent links and proces-mapping)
pub fn build_spans(jt: &JaegerTrace) -> Spans {
    if jt.data.len() != 1 {
        panic!("File contains {} (expected exactly 1)", jt.data.len());
    }
    let item = &jt.data[0]; 
    let proc_map = build_process_map(item);

    let mut spans: Vec<_> = item.spans
        .iter()
        .map(|jspan| {
            Span::new(jspan, &proc_map)
        })
        .collect();

    add_parents(&mut spans, &item.spans);
    mark_leafs(&mut spans);

    spans
}






