extern crate fitreader;
use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("fits/working_garmin.fit");
    let _ = fitreader::FitFile::read(filepath);
}
