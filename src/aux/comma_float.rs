use std::sync::Mutex;

static COMMA_FLOAT: Mutex<bool> = Mutex::new(false);

pub fn set_comma_float(val: bool) {
    let mut guard = COMMA_FLOAT.lock().unwrap();
    *guard = val
}

/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
pub fn format_float(val: f64) -> String {
    let s = format!("{}", val);
    if *COMMA_FLOAT.lock().unwrap() {
        s.replace('.', ",")
    } else {
        s
    }
}

/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
pub fn format_float_opt(val: Option<f64>) -> String {
    match val {
        Some(v) => format_float(v),
        None => "--".to_owned(),
    }
}


pub fn floats_to_string(values: Vec<Option<f64>>, sep: &str) -> String {
    values
        .into_iter()
        .map(|v| match v {
            Some(v) => format_float(v),
            None => "".to_string()
        })
        .collect::<Vec<_>>()
        .join(sep)
}
