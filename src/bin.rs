extern crate fitreader;
use fitreader::fit::Fit;
use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("fits/working_garmin.fit");
    let _ = Fit::read_file(filepath);
}
