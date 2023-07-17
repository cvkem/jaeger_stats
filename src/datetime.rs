use std::sync::Mutex;
use chrono::{
    DateTime,
//    FixedOffset,
    NaiveDateTime,
    TimeZone,
    Utc};

// UTC + 2 hours
static tz_offset: Mutex<u64> = Mutex::new(0);

pub fn set_tz_offset_minutes(minutes: u64) {
    let mut guard = tz_offset.lock().unwrap();
    *guard = minutes*60*1000*1000;
}

/// micros_to_datetime takes the number of micro-seconds since epoch (unsigned) and returns a UTC-DateTime.
pub fn micros_to_datetime(epoch_micros: u64) -> DateTime<Utc> {
//    let epoch_micros = epoch_micros + *tz_offset.lock().unwrap();
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