//! set of auxiliary functions (datetime conversion, log-reporting, file-operations and hashing)
//!
mod aggregate_data;
mod comma_float;
mod counted;
mod csv_file;
mod datetime;
mod file;
mod fs;
mod hash;
mod rate;
mod regression;
mod report;
mod time_stats;

pub use self::{
    aggregate_data::{AdditiveData, AggregateData, AverageData},
    comma_float::{
        floats_ref_to_string, floats_to_string, format_float, format_float_opt, set_comma_float,
    },
    counted::Counted,
    csv_file::CsvFileBuffer,
    datetime::{
        datetime_micros_str, datetime_millis_str, datetime_to_micros, micros_to_datetime,
        set_tz_offset_minutes,
    },
    file::{
        clean_os_string, current_folder, extend_create_folder, extend_with_base_path,
        extend_with_base_path_opt, extract_base_path, is_rooted_path, read_lines,
        write_string_to_file,
    },
    fs::canonicalize_path,
    hash::{hash, string_hash},
    rate::{calc_rate, set_show_rate_output},
    regression::ExponentialRegression,
    regression::LinearRegression,
    report::{report, write_report, Chapter},
    time_stats::TimeStats,
};
