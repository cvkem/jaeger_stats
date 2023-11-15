use crate::utils::{self, Chapter};
use regex::Regex;
use serde::{Deserialize, Serialize};

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
            _ => {
                let msg = format!("Invalid value for CallDirection. Observed: {s}");
                let ingest_msg = "Issue might be ingest issue: ".to_string() + &msg;
                utils::report(Chapter::Details, msg);
                utils::report(Chapter::Ingest, ingest_msg);
                CallDirection::Unknown
            }
        }
    }
}

impl From<Option<&String>> for CallDirection {
    fn from(s: Option<&String>) -> Self {
        match s {
            Some(s) => match &s[..] {
                "server" | "consumer" => CallDirection::Inbound,
                "client" | "producer" => CallDirection::Outbound,
                _ => {
                    let msg = format!("Invalid value for CallDirection. Observed: {s:?}");
                    let ingest_msg = "Issue might be ingest issue: ".to_string() + &msg;
                    utils::report(Chapter::Details, msg);
                    utils::report(Chapter::Ingest, ingest_msg);
                    CallDirection::Unknown
                }
            },
            None => CallDirection::Unknown,
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

    pub fn get_process(&self) -> String {
        self.process.to_owned()
    }

    /// Extract a call record from a string as generated by 'get_process_method()', assuming call_direction unknown
    pub fn extract_call(service_operation: &str) -> Option<Self> {
        let split = Regex::new(r"^([^/]+)/(.+)$").unwrap();
        let Some(caps) = split.captures(service_operation) else {
            println!("Failed to split '{service_operation}' in a service and an operations separated by a '/'");
            return None
        };
        let (_, [service, operation]) = caps.extract();
        Some(Call {
            process: service.to_string(),
            method: operation.to_string(),
            call_direction: CallDirection::Unknown,
        })
    }
}

impl ToString for Call {
    fn to_string(&self) -> String {
        match self.call_direction {
            CallDirection::Unknown => self.process.to_owned() + "/" + &self.method,
            _ => {
                self.process.to_owned()
                    + "/"
                    + &self.method
                    + " ["
                    + self.call_direction.as_str()
                    + "]"
            }
        }
    }
}
