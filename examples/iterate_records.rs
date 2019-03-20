extern crate env_logger;
extern crate fit;

use std::path::PathBuf;

fn main() {
    env_logger::init();
    let filepath = PathBuf::from("fits/garmin_520_power.fit");
    let f = fit::read(filepath).unwrap();
    for r in f.messages().filter_name("Record") {
        println!("{:#?}", r.all_values());
    }
}
