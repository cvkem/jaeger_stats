use clap::ValueEnum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum EdgeValue {
    Count,
    AvgMillis,
    MedianMillis,
    P75Millis,
    P90Millis,
    P95Millis,
    P99Millis,
    MaxMillis,
}


const COUNT: &str = "COUNT";
const AVGMILLIS: &str = "AVGMILLIS";
const MEDIANMILLIS: &str = "MEDIANMILLIS";
const P75MILLIS: &str = "P75MILLIS";
const P90MILLIS: &str = "P90MILLIS";
const P95MILLIS: &str = "P95MILLIS";
const P99MILLIS: &str = "P99MILLIS";
const MAXMILLIS: &str = "MAXMILLIS";

impl ToString for EdgeValue {
    fn to_string(&self) -> String {
        match self {
            EdgeValue::Count => COUNT.to_owned(),
            EdgeValue::AvgMillis => AVGMILLIS.to_owned(),
            EdgeValue::MedianMillis => MEDIANMILLIS.to_owned(),
            EdgeValue::P75Millis => P75MILLIS.to_owned(),
            EdgeValue::P90Millis => P90MILLIS.to_owned(),
            EdgeValue::P95Millis => P95MILLIS.to_owned(),
            EdgeValue::P99Millis => P99MILLIS.to_owned(),
            EdgeValue::MaxMillis => MAXMILLIS.to_owned(),
        }
    }
}

impl From<&str> for EdgeValue {
    fn from(s: &str) -> Self {
        match &s.to_uppercase()[..] {
            COUNT => Self::Count,
            AVGMILLIS => Self::AvgMillis,
            MEDIANMILLIS => Self::MedianMillis,
            P75MILLIS => Self::P75Millis,
            P90MILLIS => Self::P90Millis,
            P95MILLIS => Self::P95Millis,
            P99MILLIS => Self::P99Millis,
            MAXMILLIS => Self::MaxMillis,
            scope => panic!("Could not derived EdgeValue for {scope}.  Expected Full, Centered, Inbound or Outbound")
        }
    }
}
