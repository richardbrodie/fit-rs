
extern crate fit;

use std::path::PathBuf;

fn main() {
    env_logger::init();
    let filepath = PathBuf::from("fits/trainerroad.fit");
    let f = fit::read(filepath).unwrap();
    println!("{:#?}", f.message_counts());
}
