//! set of auxiliary functions (datetime conversion, log-reporting, file-operations and hashing)
//!
mod counted;
mod datetime;
mod file;
mod hash;
mod report;

pub use self::{
    counted::Counted,
    datetime::{
        datetime_micros_str, datetime_millis_str, datetime_to_micros, micros_to_datetime,
        set_tz_offset_minutes,
    },
    file::{extend_create_folder, read_lines, write_string_to_file},
    hash::{hash, string_hash},
    report::{report, write_report, Chapter},
};
