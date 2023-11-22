use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

pub struct IndentStr(Mutex<Vec<Arc<String>>>);

impl IndentStr {
    fn new() -> Self {
        let base = [Arc::new(String::new())].to_vec();
        Self(Mutex::new(base))
    }

    /// get an indent string, and fill the vector with all preceding strings if these do not exists.
    pub fn get_indent_str(&self, level: usize) -> Arc<String> {
        let mut indent_guard = self.0.lock().unwrap();
        if level >= indent_guard.len() {
            ((indent_guard.len() - 1)..level).for_each(|idx| {
                let next_str = format!("\t{}", &indent_guard[idx]);
                indent_guard.push(Arc::new(next_str));
            });
        }
        Arc::clone(&indent_guard[level])
    }
}

lazy_static! {
    pub static ref INDENT_STR: IndentStr = IndentStr::new();
}
