extern crate fit;

use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("fits/trainerroad.fit");
    let f = fit::read(filepath).unwrap();
    for r in f.messages().filter_name("Session") {
        println!("{:#?}", r.values);
    }
}
