use chrono::NaiveDateTime;
use std::sync::Mutex;

// UTC + 2 hours
static TX_OFFSET: Mutex<i64> = Mutex::new(0);

pub fn set_tz_offset_minutes(minutes: i64) {
    let mut guard = TX_OFFSET.lock().unwrap();
    *guard = minutes * 60 * 1000 * 1000;
}

/// micros_to_datetime takes the number of micro-seconds since epoch (unsigned) and returns a UTC-DateTime.
pub fn micros_to_datetime(epoch_micros: i64) -> NaiveDateTime {
    let epoch_micros = epoch_micros + *TX_OFFSET.lock().unwrap();
    NaiveDateTime::from_timestamp_micros(epoch_micros)
        .unwrap_or_else(|| panic!("Invalid time provided in micros: {epoch_micros}"))
}

/// Get the microseconds sinds epoch (corrected for UTC)
pub fn datetime_to_micros(dt: NaiveDateTime) -> i64 {
    let micros = dt.timestamp_micros();
    let offset = -1 * *TX_OFFSET.lock().unwrap();
    micros + offset
}

/// date-time string as microsecond precision
pub fn datetime_micros_str(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S.%.6f").to_string()
}

/// date-time string as millisecond precision
pub fn datetime_millis_str(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S.%.3f").to_string()
}

#[cfg(test)]
mod test {
    use super::{datetime_micros_str, micros_to_datetime};

    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn test_to_datetime() {
        // micros since epoch
        const DT1: i64 = 1689678502462000;

        // corresponding date and time
        const YEAR: i32 = 2023;
        const MONTH: u32 = 07;
        const DAY: u32 = 18;
        const HOUR: u32 = 11;
        const MINUTES: u32 = 08;
        const SECONDS: u32 = 22;
        const MILLIS: u32 = 462;

        let dt1 = micros_to_datetime(DT1);
        // dt is now based on the NaiveDateTime
        //let ndt1 = NaiveDateTime::from_timestamp_micros(DT1).unwrap();

        let actual = NaiveDate::from_ymd_opt(YEAR, MONTH, DAY)
            .unwrap()
            .and_hms_milli_opt(HOUR, MINUTES, SECONDS, MILLIS)
            .unwrap();

        //        assert_eq!(dt1, DateTime::<Utc>::from_utc(ndt1, Utc));
        assert_eq!(actual, dt1)
    }
}
