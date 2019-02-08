extern crate env_logger;
extern crate fit;

use std::path::PathBuf;

fn main() {
    env_logger::init();
    let filepath = PathBuf::from("fits/trainerroad.fit");
    let f = fit::FitFile::read(filepath);
    println!("{:#?}", f.single_message("Session").unwrap().fields());
}
