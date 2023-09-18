#![allow(non_snake_case, dead_code)]
use super::{
    process_map::{build_process_map, Process, ProcessMap},
    unify_operation::unified_operation_name,
};
use crate::{
    micros_to_datetime,
    raw::{JaegerItem, JaegerLog, JaegerSpan, JaegerTags},
    utils,
};

use chrono::NaiveDateTime;
use serde_json::Value;
use std::{collections::HashMap, iter, sync::Mutex};

static MAX_LOG_MSG_LENGTH: Mutex<usize> = Mutex::new(100);

pub fn set_max_log_msg_length(val: usize) {
    let mut guard = MAX_LOG_MSG_LENGTH.lock().unwrap();
    *guard = val
}

#[derive(Debug, Default)]
pub enum Position {
    Root,
    Parent(usize),
    #[default]
    MissingParent,
}

#[derive(Debug, Default)]
pub struct Span {
    // Process should be a reference, but that complicates things:
    //    - default is not possible
    //    - how to build a Vec (no copy)
    //    - passing lifetimes
    //pub struct Span<'a> {
    //    pub process: &'a Process,
    pub process: Option<Process>,
    pub position: Position,
    pub is_leaf: bool,
    pub rooted: bool, // does this span trace back to the real root? (default = false)
    pub span_id: String,
    pub operation_name: String,
    pub full_operation_name: Option<String>,
    pub start_dt: NaiveDateTime,
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

impl Span {
    fn new(js: &JaegerSpan, proc_map: &ProcessMap) -> Self {
        let position = Default::default();
        let span_id = js.spanID.to_owned();
        let (operation_name, full_operation_name) = unified_operation_name(&js.operationName);

        let start_dt = micros_to_datetime(js.startTime);
        let duration_micros = js.duration;
        let process = proc_map.get(&js.processID).map(|proc| proc.to_owned());
        let mut span = Span {
            position,
            span_id,
            operation_name,
            full_operation_name,
            start_dt,
            duration_micros,
            process,
            ..Default::default()
        };
        span.add_tags(&js.tags);
        span.add_logs(&js.logs);
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

    fn add_logs(&mut self, logs: &[JaegerLog]) {
        let unpack_serde_str = |v: &Value| match v {
            Value::String(s) => s.to_owned(),
            _ => panic!("Invalid type of string-field {:?}", v),
        };

        let max_msg_len = *MAX_LOG_MSG_LENGTH.lock().unwrap();

        self.logs = logs
            .iter()
            .map(|log| {
                let timestamp = log.timestamp;
                let mut level = String::new();
                let mut msg = String::new();
                log.fields.iter().for_each(|jt| match &jt.key[..] {
                    "level" => level = unpack_serde_str(&jt.value),
                    "message" => {
                        let full = unpack_serde_str(&jt.value);
                        msg = if full.len() > max_msg_len {
                            let base: String = full.chars().take(max_msg_len).collect();
                            base + "...TRUNCATED"
                        } else {
                            full
                        }
                    }
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
pub struct Spans {
    pub items: Vec<Span>,
    pub root_idx: Option<usize>,
}

#[derive(Debug)]
pub struct Log {
    pub timestamp: i64,
    pub level: String,
    pub msg: String,
}

/// add_parents adds parent-links to spans based on the information in Vec<JaegerSpan>
fn add_parents(spans: &mut Vec<Span>, jspans: &Vec<JaegerSpan>) -> Vec<String> {
    let mut missing_span_ids = Vec::new();

    iter::zip(spans, jspans).for_each(|(span, jspan)| {
        match jspan.references.len() {
            0 => span.position = Position::Root, // this is a root
            1 if jspan.references[0].spanID == jspan.spanID => span.position = Position::Root, // this is a root as it has a self-reference, i.e. references.spanID == spanID.
            1 => {
                let parentID = &jspan.references[0].spanID;
                let mut parent_found = false;
                for (idx, js) in jspans.iter().enumerate() {
                    if js.spanID[..] == parentID[..] {
                        span.position = Position::Parent(idx);
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

impl Spans {
    /// mark_leafs sets the is_leaf value of each span.
    /// A leaf-span is a span that can not be reached from any other span.
    fn mark_leafs(&mut self) {
        let mut is_leaf = Vec::with_capacity(self.items.len());
        // Default assumption is that all spans are leafs.
        (0..self.items.len()).for_each(|_| is_leaf.push(true));
        // However, spans that are reacheabe via another span are not a leaf.
        self.items.iter().for_each(|span| match span.position {
            Position::Root | Position::MissingParent => (),
            Position::Parent(par) => is_leaf[par] = false,
        });

        // And finaly update the is_leaf value of all spans
        iter::zip(self.items.iter_mut(), is_leaf)
            .for_each(|(span, is_leaf)| span.is_leaf = is_leaf);
    }

    /// Auxiliary furnction for self.mark_rooted()
    fn mark_root_path_aux(&mut self, idx: usize) -> bool {
        if self.items[idx].rooted {
            true
        } else {
            match self.items[idx].position {
                Position::Parent(parent) => {
                    let rooted = self.mark_root_path_aux(parent);
                    self.items[idx].rooted = rooted;
                    rooted
                }
                Position::Root => {
                    self.items[idx].rooted = true;
                    true
                }
                Position::MissingParent => false,
            }
        }
    }

    /// Mark all spans that are connected to the root with rooted=true.
    fn mark_rooted(&mut self) {
        // TODO next line is not needed as it is handled already.
        if let Some(root_idx) = self.root_idx {
            self.items[root_idx].rooted = true;

            (0..self.items.len()).for_each(|idx| {
                self.mark_root_path_aux(idx);
            });
        }
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

        let roots: Vec<_> = spans
            .iter()
            .enumerate()
            .filter_map(|(idx, span)| match span.position {
                Position::Root => Some(idx),
                _ => None,
            })
            .collect();
        let mut spans = if roots.len() == 1 {
            Spans {
                items: spans,
                root_idx: Some(*roots.first().unwrap()),
            }
        } else {
            let issue = format!(
                "Found a trace '{}'with {} roots (expected exactly 1 root)\n\tfull trace has {} spans.",
                item.traceID,
                roots.len(),
                item.spans.len(),
            );
            utils::report(crate::utils::Chapter::Issues, issue);
            Spans {
                items: spans,
                root_idx: None, // assume a default
            }
        };

        spans.mark_leafs();

        spans.mark_rooted();

        (spans, missing_span_ids)
    }

    /// chain_apply_forward is used to run over a call-chain and apply the 'process' to each span in order to get a Vec<T>
    pub fn chain_apply_forward<T>(&self, idx: usize, process: &dyn Fn(&Span) -> T) -> Vec<T> {
        //        let chain_apply_forward_aux = |
        assert!(
            idx < self.items.len(),
            "Provided index exceeds the available spans"
        );
        let span = &self.items[idx];
        // find the root and allocate vector
        let mut result = match span.position {
            Position::Root => Vec::new(),
            Position::MissingParent => Vec::new(), // we chousl carry a flag rooted=false along. However, for current use-case not (yet) needed.
            Position::Parent(idx) => self.chain_apply_forward(idx, process),
        };
        let ret = process(span);
        result.push(ret);
        //        result.push(process(span));
        result
    }
}
