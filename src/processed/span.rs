#![allow(non_snake_case)]
use super::process_map::{build_process_map, Process, ProcessMap};
use crate::{
    micros_to_datetime,
    raw::{JaegerItem, JaegerLog, JaegerSpan, JaegerTags},
};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::{collections::HashMap, iter};

#[derive(Debug, Default)]
pub struct Span {
    // Process should be a reference, but that complicates things:
    //    - default is not possible
    //    - how to build a Vec (no copy)
    //    - passing lifetimes
    //pub struct Span<'a> {
    //    pub process: &'a Process,
    pub process: Option<Process>,
    pub parent: Option<usize>,
    pub is_leaf: bool,
    pub rooted: bool, // does this span trace back to the real root? (default = false)
    pub span_id: String,
    pub operation_name: String,
    pub full_operation_name: Option<String>,
    pub start_dt: DateTime<Utc>,
    pub duration_micros: i64,
    // optional parameters from tags
    // to see statistics on all tags run:
    //      cargo run --example collect_span_tags
    //
    pub span_kind: Option<String>,
    pub http_status_code: Option<i16>,
    pub attributes: HashMap<String, String>,
    // some attributes
    // pub http_method: Option<String>,
    // pub http_url: Option<String>,
    // pub component: Option<String>,
    // pub db_instance: Option<String>,
    // pub db_type: Option<String>,
    // pub db_statement: Option<String>,
    // pub warnings: Option<Vec<String>>,
    pub logs: Vec<Log>,
}

pub type Spans = Vec<Span>;

impl Span {
    fn new(js: &JaegerSpan, proc_map: &ProcessMap) -> Self {
        let parent = None;
        let span_id = js.spanID.to_owned();
        let (operation_name, full_operation_name) = get_operation_unified(&js.operationName);

        let start_dt = micros_to_datetime(js.startTime);
        let duration_micros = js.duration;
        let process = proc_map.get(&js.processID).map(|proc| proc.to_owned());
        let mut span = Span {
            parent,
            span_id,
            operation_name,
            full_operation_name,
            start_dt,
            duration_micros,
            process,
            ..Default::default()
        };
        span.add_tags(&js.tags);
        span
    }

    /// two attributes are extracted as these are used frequently, the others are stored in a hashmap
    fn add_tags(&mut self, tags: &JaegerTags) {
        tags.iter().for_each(|tag| match &tag.key[..] {
            "http.status_code" => self.http_status_code = Some(tag.get_i16()),
            "span.kind" => self.span_kind = Some(tag.get_string()),
            key => _ = self.attributes.insert(key.to_owned(), tag.get_as_string()),
        });
        // tags.iter().for_each(|tag| match &tag.key[..] {
        //     "span.kind" => self.span_kind = Some(tag.get_string()),
        //     "http.status_code" => self.http_status_code = Some(tag.get_i32()),
        //     "http.method" => self.http_method = Some(tag.get_string()),
        //     "http.url" => self.http_url = Some(tag.get_string()),
        //     "component" => self.component = Some(tag.get_string()),
        //     "db.instance" => self.db_instance = Some(tag.get_string()),
        //     "db.type" => self.db_instance = Some(tag.get_string()),
        //     "db.statement" => self.db_statement = Some(tag.get_string()),
        //     "identity.eb_contract_id " | "eb_contract" => self.eb_contract = Some(tag.get_string()),
        //     _ => (),
        // })
    }

    fn add_logs(&mut self, logs: &Vec<JaegerLog>) {
        let unpack_serde_str = |v: &Value| match v {
            Value::String(s) => s.to_owned(),
            _ => panic!("Invalid type of string-field {:?}", v),
        };

        self.logs = logs
            .iter()
            .map(|log| {
                let timestamp = log.timestamp;
                let mut level = String::new();
                let mut msg = String::new();
                log.fields.iter().for_each(|jt| match &jt.key[..] {
                    "level" => level = unpack_serde_str(&jt.value),
                    "message" => msg = unpack_serde_str(&jt.value),
                    _ => (),
                });
                Log {
                    timestamp,
                    level,
                    msg,
                }
            })
            .collect();
    }

    //. get_process_name returns the string-slice of the process of this span (without the operation (method) that is called)
    pub fn get_process_str(&self) -> &str {
        match &self.process {
            Some(p) => &p.name[..],
            None => "-",
        }
    }

    // //. get_process_name returns the name of the process of this span (without the operation (method) that is called)
    // pub fn get_process_name(&self) -> String {
    //     self.get_process_str().to_owned()
    // }
}

#[derive(Debug)]
pub struct Log {
    pub timestamp: i64,
    pub level: String,
    pub msg: String,
}

fn replace_regex(s: String, re: &Regex, replacement: &str) -> (String, bool) {
    let tmp = s.clone(); // TODO: this clone is usedless (just to satisfy the borrow-checker)
    let found: Vec<&str> = re.find_iter(&tmp).map(|m| m.as_str()).collect();
    if found.len() > 0 {
        let s = found
            .into_iter()
            .fold(s, |s, obs| s.replace(obs, replacement));
        (s, true)
    } else {
        (s, false)
    }
}

fn get_operation_unified(js_operation: &str) -> (String, Option<String>) {
    lazy_static! {
        static ref RE_REK: Regex = Regex::new(r"/\d{6,10}").unwrap();  // typically 8-10
        // should be a T{0,1}  however, that is inconsistency in source
        static ref RE_TIME: Regex = Regex::new(r"(?x)
            /T\d{4}-\d{2}-\d{2}_
            \d{5,10}").unwrap();
        static ref RE_SPAAR: Regex = Regex::new(r"(?x)
            /[0-9a-f]{8}-
            [0-9a-f]{4}-
            [0-9a-f]{4}-
            [0-9a-f]{4}-
            [0-99-f]{12}").unwrap();
        static ref RE_BASE: Regex = Regex::new(r"(?x)
            /[a-zA-Z0-9\-_]{39,40}
            ={0,1}
            /").unwrap();
        static ref RE_JAARREK: Regex = Regex::new(r"\-\d{5,9}\-20\d{2}").unwrap();
    }

    let oper_name = js_operation.to_owned();
    let (oper_name, repl_time) = replace_regex(oper_name, &RE_TIME, "/{TIME}");
    let (oper_name, repl_spaarpot) = replace_regex(oper_name, &RE_SPAAR, "/{SPAAR}");
    let (oper_name, repl_base) = replace_regex(oper_name, &RE_BASE, "/{BASE}/");
    let (oper_name, repl_jaarrek) = replace_regex(oper_name, &RE_JAARREK, "-{JAARREK}");
    // do next one last, as it might conflict with others.
    let (oper_name, repl_rek) = replace_regex(oper_name, &RE_REK, "/{ACCOUNT}");

    if repl_rek || repl_time || repl_spaarpot || repl_base || repl_jaarrek {
        (oper_name, Some(js_operation.to_owned()))
    } else {
        (oper_name, None)
    }
}

/// mark_leafs sets the is_leaf value of each span.
fn mark_leafs(spans: &mut Spans) {
    let mut is_leaf = Vec::with_capacity(spans.len());
    (0..spans.len()).for_each(|_| is_leaf.push(true));
    spans.iter().for_each(|span| match span.parent {
        None => (),
        Some(par) => is_leaf[par] = false,
    });

    iter::zip(spans, is_leaf).for_each(|(mut span, is_leaf)| span.is_leaf = is_leaf);
}

/// add_parents adds parent-links to spans based on the information in Vec<JaegerSpan>
fn add_parents(spans: &mut Spans, jspans: &Vec<JaegerSpan>) -> Vec<String> {
    let iter = iter::zip(spans, jspans);

    let mut missing_span_ids = Vec::new();

    iter.for_each(|(span, jspan)| {
        match jspan.references.len() {
            0 => (), // this is the root
            1 => {
                let parentID = &jspan.references[0].spanID;
                let mut parent_found = false;
                for (idx, js) in jspans.iter().enumerate() {
                    if js.spanID[..] == parentID[..] {
                        span.parent = Some(idx);
                        parent_found = true;
                        break;
                    }
                }
                if !parent_found {
                    missing_span_ids.push(parentID.to_owned());
                }
            }
            num => panic!("Span '{}' has {num} parent-references.", jspan.spanID),
        }
    });
    missing_span_ids
}

/// mark all spans that are connected to the root.
fn mark_rooted(spans: &mut Spans) {
    if spans[0].parent.is_some() {
        println!(
            "Could not find root at index=0, as it has parent {}",
            spans[0].parent.unwrap()
        );
        return;
    }
    spans[0].rooted = true;

    fn mark_root_path(spans: &mut Spans, idx: usize) -> bool {
        if spans[idx].rooted {
            return true;
        }
        if let Some(parent) = spans[idx].parent {
            let rooted = mark_root_path(spans, parent);
            spans[idx].rooted = rooted;
            return rooted;
        } else {
            return false;
        }
    }

    (0..spans.len()).for_each(|idx| {
        mark_root_path(spans, idx);
    });
}

/// build the list of spans (including parent links and proces-mapping)
pub fn build_spans(item: &JaegerItem) -> (Spans, Vec<String>) {
    let proc_map = build_process_map(item);

    let mut spans: Vec<_> = item
        .spans
        .iter()
        .map(|jspan| Span::new(jspan, &proc_map))
        .collect();

    let missing_span_ids = add_parents(&mut spans, &item.spans);
    mark_leafs(&mut spans);

    mark_rooted(&mut spans);

    (spans, missing_span_ids)
}
