use serde::{Deserialize, Serialize};
use serde_json;



#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerProcess {
    serviceName: String,
    tags: Vec<JaegerTag>,
}
    

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerLog {
    timestamp: u64,
    fields: Vec<JaegerTag>,
}
    

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerReference {
    refType: String,
    traceID: String,
    spanID: String,
}
    
#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerTag {
    key: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    type_id: String,
    value: serde_json::Value
}
    
#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerSpan {
    traceID: String,
    spanID: String,
    flags: i32,
    operationName: String,
    references: Vec<JaegerReference>,
    startTime: u64,
    duration: u64,
    tags: Vec<JaegerTag>,
    logs: Vec<JaegerLog>,
    processID: String,
    warnings: Option<Vec<String >>,
}
    
#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerItem {
    pub traceID: String,
    pub spans: Vec<JaegerSpan>,
    // processes: serde_json::Map<String, JaegerProcess>,
    // ERROR: the trait `Deserialize<'_>` is not implemented for `serde_json::Map<std::string::String, JaegerProcess>`
    pub processes: serde_json::Map<String, serde_json::Value>,
    pub warnings: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerError {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerTrace {
    pub data: Vec<JaegerItem>,
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
    pub errors: Option<Vec<JaegerError>>,
}



