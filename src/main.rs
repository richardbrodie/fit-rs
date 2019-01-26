extern crate fit;
use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("fits/working_garmin.fit");
    let _ = fit::FitFile::read(filepath);
}
