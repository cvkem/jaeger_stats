use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TraceScope {
    All,
    End2end,
    Inbound,
    //    Outbound,
}

const ALL: &str = "ALL";
const END2END: &str = "END2END";
const INBOUND: &str = "INBOUND";

impl ToString for TraceScope {
    fn to_string(&self) -> String {
        match self {
            TraceScope::All => ALL.to_owned(),
            TraceScope::End2end => END2END.to_owned(),
            TraceScope::Inbound => INBOUND.to_owned(),
        }
    }
}

impl TryFrom<&str> for TraceScope {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match &s.to_uppercase()[..] {
            ALL => Ok(Self::All),
            END2END => Ok(Self::End2end),
            INBOUND => Ok(Self::Inbound),
            _ => Err("Could not derive TraceScope from string.  Expected Inbound, End2end or All."),
        }
    }
}
