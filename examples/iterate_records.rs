extern crate fit;

use std::path::PathBuf;
use fit::MessageType;

fn main() {
    let filepath = PathBuf::from("data/garmin_1000.fit");
    let msgs = fit::run(&filepath);
    for r in msgs.iter().filter(|m| m.kind == MessageType::Record) {
        println!("{:?}", r.values);
    }
}
