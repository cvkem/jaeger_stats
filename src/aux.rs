use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, Write},
    path::Path};


// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path> +std::fmt::Debug + Copy, {
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
