use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessListItem {
    pub idx: i64,
    pub key: String,
    pub display: String, // the display can be used in select-boxes, but is nog guaranteed to be unique (at least nog for a Call-chain list.)
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
