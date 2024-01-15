use clap::ValueEnum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum MermaidScope {
    Full,
    Centered,
    Inbound,
    Outbound,
}

const FULL: &str = "FULL";
const CENTERED: &str = "CENTERED";
const INBOUND: &str = "INBOUND";
const OUTBOUND: &str = "OUTBOUND";

impl ToString for MermaidScope {
    fn to_string(&self) -> String {
        match self {
            MermaidScope::Full => FULL.to_owned(),
            MermaidScope::Centered => CENTERED.to_owned(),
            MermaidScope::Inbound => INBOUND.to_owned(),
            MermaidScope::Outbound => OUTBOUND.to_owned(),
        }
    }
}

impl From<&str> for MermaidScope {
    fn from(s: &str) -> Self {
        match &s.to_uppercase()[..] {
            FULL => Self::Full,
            CENTERED => Self::Centered,
            INBOUND => Self::Inbound,
            OUTBOUND => Self::Outbound,
            scope => panic!("Could not derived MermaidScope for {scope}.  Expected Full, Centered, Inbound or Outbound")
        }
    }
}
