//use crate::{aux::read_lines, cchain_cache::EndPointCChain, cchain_stats::CChainStatsKey};
use serde::{Deserialize, Serialize};
//use std::{error::Error, path::Path};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Clone)]
pub enum CallDirection {
    Inbound,
    Outbound,
    #[default]
    Unknown,
}

impl From<&str> for CallDirection {
    fn from(s: &str) -> Self {
        match s {
            "Inbound" => CallDirection::Inbound, // would be nice if "Inbound" could be taken from 'CallDirection::Inbound.as_str()'
            "Outbound" => CallDirection::Outbound,
            "Unknown" => CallDirection::Unknown,
            _ => panic!("Invalid value for CallDirection"),
        }
    }
}

impl From<Option<&String>> for CallDirection {
    fn from(s: Option<&String>) -> Self {
        match s {
            Some(s) if &s[..] == "server" => CallDirection::Inbound,
            Some(s) if &s[..] == "client" => CallDirection::Outbound,
            None => CallDirection::Unknown,
            _ => panic!("Invalid value for CallDirection: Observed '{s:?}'"),
        }
    }
}

impl CallDirection {
    fn as_str(&self) -> &'static str {
        match self {
            CallDirection::Inbound => "Inbound",
            CallDirection::Outbound => "Outbound",
            CallDirection::Unknown => "Unknown",
        }
    }
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Call {
    pub process: String,
    pub method: String,
    #[serde(default)]
    pub call_direction: CallDirection,
}

impl Call {
    pub fn get_process_method(&self) -> String {
        let process = &self.process;
        let method = &self.method;
        format!("{process}/{method}")
    }
}

impl ToString for Call {
    fn to_string(&self) -> String {
        match self.call_direction {
            CallDirection::Unknown => self.process.to_owned() + "/" + &self.method,
            _ => self.process.to_owned() + "/" + &self.method + " [" + self.call_direction.as_str(),
        }
    }
}

pub type CallChain = Vec<Call>;

