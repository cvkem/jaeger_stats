use chrono::{
    DateTime,
    NaiveDateTime,
    Utc,
};
use std::sync::Mutex;

// UTC + 2 hours
static TX_OFFSET: Mutex<i64> = Mutex::new(0);

pub fn set_tz_offset_minutes(minutes: i64) {
    let mut guard = TX_OFFSET.lock().unwrap();
    *guard = minutes * 60 * 1000 * 1000;
}

/// micros_to_datetime takes the number of micro-seconds since epoch (unsigned) and returns a UTC-DateTime.
pub fn micros_to_datetime(epoch_micros: i64) -> DateTime<Utc> {
    let epoch_micros = epoch_micros + *TX_OFFSET.lock().unwrap();
    let timestamp = epoch_micros / 1000 / 1000;
    let micros = epoch_micros - timestamp * 1000 * 1000;

    // Create a NaiveDateTime from the timestamp
    let naive =
        NaiveDateTime::from_timestamp_opt(timestamp as i64, (micros as u32) * 1000).unwrap();
    // Create a normal DateTime from the NaiveDateTime
    DateTime::from_utc(naive, Utc)
}

/// Get the microseconds sinds epoch (corrected for UTC)
pub fn datetime_to_micros(dt: DateTime<Utc>) -> i64 {
    let micros = dt.timestamp_micros();
    let offset = -1 * *TX_OFFSET.lock().unwrap();
    micros + offset
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


