use jaeger_stats::{
    datetime_micros_str, micros_to_datetime, datetime_millis_str, datetime_to_micros, set_tz_offset_minutes
};


const DT1: u64 = 1689678502462000;
const DT1_STR: &str = "20230717-1122";
const TZ_OFFS_MIN: i

pub fn main() {
    set_tz_offset_minutes(2*60);
    let dt1 = micros_to_datetime(DT1);
    let dt1_str = datetime_micros_str(dt1);
    println!(" integer {DT1} translates to:\n\t{dt1}\n\tor as string {dt1_str}\n\texpected {DT1_STR}", );
    let val = datetime_to_micros(dt1);
    println!("Getting back the value returns {val}\n\t difference is {}", DT1 as i64 - val);

    println!("\nso values do NOT need a correction");
}