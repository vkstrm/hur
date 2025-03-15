use std::io::Read;
use std::path::PathBuf;

use crate::error::Error;

pub fn read_file(path: &PathBuf) -> Result<String, Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}
