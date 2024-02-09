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

impl TryFrom<&str> for MermaidScope {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match &s.to_uppercase()[..] {
            FULL => Ok(Self::Full),
            CENTERED => Ok(Self::Centered),
            INBOUND => Ok(Self::Inbound),
            OUTBOUND => Ok(Self::Outbound),
            // TODO: find solution to generate static strings from a normal string (with cache-lookup to prevent duplicates)
            //scope => Err(&format!("Could not derived MermaidScope for {scope}.  Expected Full, Centered, Inbound or Outbound"))
            _scope => Err("Could not derive MermaidScope for input.  Expected Full, Centered, Inbound or Outbound")
        }
    }
}
