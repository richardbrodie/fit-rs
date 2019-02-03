extern crate env_logger;
extern crate fit;

use std::path::PathBuf;

fn main() {
    env_logger::init();
    let filepath = PathBuf::from("fits/working_garmin.fit");
    // let filepath = PathBuf::from("fits/2913547417.fit");
    let f = fit::FitFile::read(filepath);
}
