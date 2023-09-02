use crate::utils::write_string_to_file;
use std::path::Path;

#[derive(Default)]
pub struct CsvFileBuffer {
    buffer: Vec<String>,
    start_toc: usize,
    toc_index: usize,
}

impl CsvFileBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_lines(&self) -> usize {
        self.buffer.len()
    }

    /// Adding empty lines to the buffer
    pub fn add_empty_lines(&mut self, num: usize) {
        (0..num).for_each(|_| self.buffer.push(String::new())) // empty lines translate to newlines
    }

    /// add a table of contents and result 'max_entries'  space
    pub fn add_toc(&mut self, max_entries: usize) {
        self.start_toc = self.num_lines();
        self.toc_index = 1; // entry 0 is for the title
        self.buffer
            .push("Table of Contents of this file (starting rows of sections):".to_owned());
        self.add_empty_lines(max_entries);
    }

    /// Add a section prefixed by a few empty lines, and also add the section to the table of contents, assuming space was reserved
    pub fn add_section(&mut self, title: &str) {
        self.add_empty_lines(2);
        self.buffer.push(format!("## {title}"));
        self.buffer[self.start_toc + self.toc_index] =
            format!("{:3} @ row {}: {title}", self.toc_index, self.num_lines());
        self.toc_index += 1;
    }

    /// Append all string_data to the buffer by moving it.
    pub fn append(&mut self, string_data: &mut Vec<String>) {
        self.buffer.append(string_data);
    }

    //// Add a single string to the file
    pub fn add_line(&mut self, line: String) {
        self.buffer.push(line);
    }

    /// write the data to file and drop it
    pub fn write_file(self, path: &Path) {
        match write_string_to_file(path.to_str().unwrap(), self.buffer.join("\n")) {
            Ok(()) => (),
            Err(err) => println!(
                "Writing file '{}' failed with Error: {err:?}",
                path.display()
            ),
        }
    }
}
