use std::path::{Path, PathBuf};

/// Canonilize Path and change empty path to current folder
pub fn canonicalize_path(folder: &Path) -> PathBuf {
    let folder = if folder == Path::new("") {
        Path::new(".")
    } else {
        folder
    };
    folder
        .to_path_buf()
        .canonicalize()
        .expect("Failed to make canonical path. Path probably does not exist!")
}
