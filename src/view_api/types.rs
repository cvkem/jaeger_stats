use serde::Serialize;

pub type ServiceOperString = String;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessListItem {
    pub idx: i64,
    pub key: String, // the key is a full key that ends in a specific Service/Operation
    pub display: ServiceOperString, // the display can be used in select-boxes, but is nog guaranteed to be unique (at least not for a Call-chain list.)
    pub rank: f64,
    pub avg_count: i64,
    pub chain_type: String,
    pub inbound_idx: i64,
}

pub type ProcessList = Vec<ProcessListItem>;

#[derive(Serialize, Debug)]
pub struct ChartLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
}

#[derive(Serialize, Debug)]
pub struct ChartDataParameters {
    pub title: String,
    pub process: String,
    pub metric: String,
    pub description: Vec<(String, String)>,
    pub labels: Vec<String>,
    pub lines: Vec<ChartLine>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub column_labels: Vec<String>,
    pub data: Vec<ChartLine>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SelectLabel {
    pub idx: i64, // could be u64, but will be used in json, so will be signed anyway
    pub label: String,
    pub selected: bool,
}

pub type Selection = Vec<SelectLabel>;
