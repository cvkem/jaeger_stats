//! set of auxiliary functions (datetime conversion, log-reporting, file-operations and hashing)
//! 
mod datetime;
mod file;
mod hash;
mod report;


pub use self::{
    datetime::{datetime_micros_str, datetime_millis_str, micros_to_datetime, set_tz_offset_minutes},
    file::{extend_create_folder, write_string_to_file, read_lines},
    hash::{hash, string_hash},
    report::{Chapter, report, write_report}
    };

