use std::{
    error::Error,
    ffi::OsString,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path> + std::fmt::Debug + Copy,
{
    //let file = File::open(filename)?;
    //    let fname_dbg = filename.clone();
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to open file '{filename:?}' with error: {err:?}"); //, filename.to_str()) Path::new(&filename).display());
                                                                                // can not make a descent error as AsRef<Path> is not a path and not a string.
            Err(err)?
        }
    };
    Ok(io::BufReader::new(file).lines())
}

pub fn write_string_to_file(filename: &str, data: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// create a sub-folder if it does not exist yet and return the path to this sub-folder
pub fn extend_create_folder(folder: &Path, subfolder: &str) -> PathBuf {
    let mut ext_folder = folder.to_path_buf();
    ext_folder.push(subfolder);
    if !ext_folder.is_dir() {
        fs::create_dir(ext_folder.clone()).expect("failed to create folder");
    }
    ext_folder
}

/// extract the base-path from a path containing a file, or the parent of a folder.
pub fn extract_base_path(path: &Path) -> PathBuf {
    path
        .canonicalize()
        .unwrap_or_else(|err| panic!("Failed to find the canonical path. Path '{}' probably does not exist!\n\tError: {err}", path.display()))
        .parent()
        .expect("Could not extract base_path of stitch-list")
        .to_path_buf()
}

pub fn current_folder() -> PathBuf {
    Path::new(".").canonicalize().unwrap().to_path_buf()
}

/// check if is path is a root-path (starts with '/' or '\')
pub fn is_rooted_path(path: &str) -> bool {
    path.starts_with('/') || path.starts_with('\\')
}

/// Remove end-of-line comments and trim whitespace
fn clean_path_str(path: &str) -> &str {
    match path.find('#') {
        Some(pos) => path[0..pos].trim(),
        None => path,
    }
}

/// Translate a str to a clean OsString
pub fn clean_os_string(path: &str) -> OsString {
    clean_path_str(path).to_string().into()
}

/// extend a path with a base-path. However, path is not allowed to be an absolute path (rooted-path) as these can not be extended.
pub fn extend_with_base_path(base_path: &Path, path: &str) -> OsString {
    if is_rooted_path(path) {
        panic!(
            "Can not extend a path that starts with '{}'",
            path.chars().next().unwrap()
        );
    };
    extend_with_base_path_opt(base_path, path)
}

/// extend a path with a base-path
pub fn extend_with_base_path_opt(base_path: &Path, path: &str) -> OsString {
    // skip comments at the tail of the path-string
    let mut path = clean_path_str(path);

    if is_rooted_path(path) {
        // absolute paths (rooted paths) can not be extended
        path.to_string().into()
    } else {
        // correct base-path for ".." on path
        let mut base_path_buf = base_path.to_path_buf();
        while path.starts_with("../") || path.starts_with(r"..\") {
            path = &path[3..];
            if !base_path_buf.pop() {
                panic!(
                    "can not backtrack via .. beyond the root basepath {base_path:?} for path {path}"
                );
            }
        }

        base_path_buf.push(Path::new(path));
        println!("base_path now is {base_path_buf:?}");
        base_path_buf
            .canonicalize()
            .map_err(|err| {
                eprintln!("\nFailed to handle path {base_path:?}. File probably does not exist!!");
                err
            })
            .unwrap();

        base_path_buf.into_os_string()
    }
}
