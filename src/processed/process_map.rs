use crate::raw::JaegerItem;
use serde_json::Value;
use std::collections::HashMap;

const SHOW_STDOUT: bool = false;

#[derive(Debug, Default, Clone)]
pub struct Process {
    pub name: String,
    pub server_name: String,
    pub ip: String,
    pub jaeger_version: String,
}

impl Process {
    /// Extend the Process with a servername from Json
    fn with_servername(&mut self, data: &Value) {
        let Value::String(name) = data else {panic!("Expected servicename to be a String")};
        self.name = name.to_owned();
    }

    /// Extend a Process with tags data
    fn with_tags(&mut self, proc_key: &String, data: &Value) {
        match data {
            // expect an array of tags
            Value::Array(val) => {
                for tag in val.iter() {
                    let Value::String(key) = tag.get("key").unwrap() else { panic!("key is not a string"); };
                    let val = if let Value::String(val) = tag.get("value").unwrap() {
                        val.to_owned()
                    } else {
                        panic!("key is not a string");
                    };
                    match &key[..] {
                        "hostname" => self.server_name = val,
                        "ip" => self.ip = val,
                        "jaeger.version" => self.jaeger_version = val,
                        _ => panic!("Found unknown key '{key}' for process {proc_key}"),
                    }
                }
            }
            _ => panic!("Expected tags-array, but found '{data}' for process '{proc_key}'."),
        }
    }
}

pub type ProcessMap = HashMap<String, Process>;

/// Build_process takes a JaegerItem and extract a mapping from keys like 'p2' to a Process-structs.
/// The nested structure of JSON items with flexible key-value pairs is flattened to simple Struct for convenient access downstream (during processing)
/// (This is the imperative version, next version is in functional style)
#[allow(dead_code)]
fn build_process_map_imperative(item: &JaegerItem) -> ProcessMap {
    let mut proc_map = HashMap::new();

    for (proc_key, val) in &item.processes {
        let mut proc: Process = Default::default();

        match val {
            Value::Object(val) => {
                // now unpack the object as a series of key-value pairs
                for (key2, val2) in val {
                    match &key2[..] {
                        "serviceName" => proc.with_servername(val2),
                        "tags" => proc.with_tags(proc_key, val2),
                        _ => panic!("Unexpected key for process {proc_key}: '{key2}'"),
                    }
                }
            }
            _ => panic!("Expected process {proc_key} to refer to an object. Found {val}"),
        }
        if SHOW_STDOUT {
            println!("Insert Proc {proc:?}");
        }
        // Insert the extracted process
        proc_map.insert(proc_key.to_owned(), proc);
    }

    proc_map
}

/// Build_process takes a JaegerItem and extract a mapping from keys like 'p2' to a Process-structs.
/// The nested structure of JSON items with flexible key-value pairs is flattened to simple Struct for convenient access downstream (during processing)
/// (this is the functional version, the imperative version is called 'build_process_map_imperative')
pub fn build_process_map(item: &JaegerItem) -> ProcessMap {
    item.processes
        .iter()
        .map(|(proc_key, val)| {
            let mut proc: Process = Default::default();

            match val {
                Value::Object(val) => {
                    // now unpack the object as a series of key-value pairs
                    for (key2, val2) in val {
                        match &key2[..] {
                            "serviceName" => proc.with_servername(val2),
                            "tags" => proc.with_tags(proc_key, val2),
                            _ => panic!("Unexpected key for process {proc_key}: '{key2}'"),
                        }
                    }
                }
                _ => panic!("Expected process {proc_key} to refer to an object. Found {val}"),
            }
            let proc_kv = (proc_key.to_owned(), proc);
            if SHOW_STDOUT {
                println!(" extracted process: {proc_kv:?}");
            }
            proc_kv
        })
        .collect()
}
