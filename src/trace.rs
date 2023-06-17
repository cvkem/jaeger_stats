use std::collections::HashMap;
use crate::{JaegerTrace, JaegerItem};

#[derive(Debug, Default)]
pub struct Process {
    pub name: String,
    pub server_name: String,
    pub ip: String
}

type ProcessMap = HashMap<String, Process>;

/// Build_process takes a JaegerItem and extract a mapping from keys like 'p2' to a Process-structs.
/// The nested structure of JSON items with flexible key-value pairs is flattened to simple Struct for convenient access downstream (during processing) 
fn build_process_map(item: &JaegerItem) ->  ProcessMap {
    let mut proc_map = HashMap::new();

    for (key, val) in item.processes {
        let mut proc: Process = Default::default();

        match val {
            serde_json::Value::Object(val) {
                // now unpack the object as a series of key-value pairs
                for (key2, val2) in val {
                    match &key2 {
                        "serviceName" => {
                            proc.name = key2.to_owned()
                        },
                        "tags" => {
            
                        },
                        _ => {
                            panic!("Unexpected key for process {key}: '{key2}'");
                        }
                    }    
                }
        
            },
            _ => panic!("Expected process {key} to refer to an object. Found {val}")
        }
        // Insert the extracted process
        proc_map.insert(key, proc)
    };

    proc_map
}


pub fn test_trace(jt: &JaegerTrace) -> ProcessMap {
    for item in jt.data.iter() {
        println!(" Found trace: {}", item.traceID);
        let proc_map = build_process_map(item);

        println!("{proc_map:#?}");
    };
    build_process_map(jt)
}