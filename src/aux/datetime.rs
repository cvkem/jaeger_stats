use chrono::{DateTime, NaiveDateTime, Utc};
use std::sync::Mutex;

// UTC + 2 hours
static TX_OFFSET: Mutex<i64> = Mutex::new(0);

pub fn set_tz_offset_minutes(minutes: i64) {
    let mut guard = TX_OFFSET.lock().unwrap();
    *guard = minutes * 60 * 1000 * 1000;
}

///TODO: consider moving to NaiveTime as time-zone informtion is not really present.
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

#[cfg(test)]
mod test {
    use super::{datetime_micros_str, micros_to_datetime};

    use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

    const DT1: i64 = 1689678502462000;
    const DT1_STR: &str = "20230718-1122";

    // 2023-07-18T11:08:22.462Z
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
        let ndt1 = NaiveDateTime::from_timestamp_micros(DT1).unwrap();

        let actual = NaiveDate::from_ymd_opt(YEAR, MONTH, DAY)
            .unwrap()
            .and_hms_milli_opt(HOUR, MINUTES, SECONDS, MILLIS)
            .unwrap();

        let dt1_str = datetime_micros_str(dt1);
        println!("  dt1={:?}  ndt1={:?}   {dt1_str}", dt1, ndt1);


        assert_eq!(dt1, DateTime::<Utc>::from_utc(ndt1, Utc));
        assert_eq!(actual, ndt1)
    }
}
