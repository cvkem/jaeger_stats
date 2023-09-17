//use std::fs::File;
use std::sync::Mutex;

/// Keeping track of all file-names in used and mapping them to an index
pub struct FileTracker {
    files: Vec<String>,
}

impl FileTracker {
    /// add a file to the tracker
    pub fn add_file(&mut self, file_name: String) {
        self.files.push(file_name)
    }

    pub fn get_last_idx(&self) -> usize {
        assert!(
            !self.files.is_empty(),
            "Add at least one filename before getting an index"
        );
        self.files.len() as usize - 1
    }

    pub fn get_file_name(&self, idx: usize) -> String {
        assert!(idx < self.files.len(), "Index out of bounds");
        self.files[idx].to_owned()
    }
}

pub static FILE_TRACKER: Mutex<FileTracker> = Mutex::new(FileTracker { files: Vec::new() });
