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
//const OUTBOUND: &str = "OUTBOUND";

impl ToString for TraceScope {
    fn to_string(&self) -> String {
        match self {
            TraceScope::All => ALL.to_owned(),
            TraceScope::End2end => END2END.to_owned(),
            TraceScope::Inbound => INBOUND.to_owned(),
            //            TraceScope::Outbound => OUTBOUND.to_owned(),
        }
    }
}

impl From<&str> for TraceScope {
    fn from(s: &str) -> Self {
        match &s.to_uppercase()[..] {
            ALL => Self::All,
            END2END => Self::End2end,
            INBOUND => Self::Inbound,
            //           OUTBOUND => Self::Outbound,
            scope => panic!(
                "Could not derived TraceScope for {scope}.  Expected Inbound, End2end or All."
            ),
        }
    }
}
