use chrono::{
    DateTime,
    NaiveDateTime,
    Utc,
    TimeZone};

/// micros_to_datetime takes the number of micro-seconds since epoch (unsigned) and returns a DateTime adjusted for timezone.
pub fn micros_to_datetime(epoch_micros: u64) -> DateTime<Utc> {
    let timestamp = epoch_micros/1000/1000;
    let micros = epoch_micros - timestamp * 1000 * 1000;
   
    // Create a NaiveDateTime from the timestamp
    let naive = NaiveDateTime::from_timestamp_opt(timestamp as i64, (micros as u32)*1000).unwrap();
    // Create a normal DateTime from the NaiveDateTime
    DateTime::from_utc(naive, Utc)
}


/// date-time string as microsecond precision
pub fn datetime_micros_str(dt: DateTime<Utc>) -> String {
    let newdate = dt.format("%Y-%m-%d %H:%M:%S.%.6f");
    format!("{}", newdate)
}


/// date-time string as millisecond precision
pub fn datetime_millis_str(dt: DateTime<Utc>) -> String {
    let newdate = dt.format("%Y-%m-%d %H:%M:%S.%.3f");
    format!("{}", newdate)
}