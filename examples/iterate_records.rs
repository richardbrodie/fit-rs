extern crate env_logger;
extern crate fit;

use std::path::PathBuf;

fn main() {
    env_logger::init();
    let filepath = PathBuf::from("fits/garmin_520_power.fit");
    let f = fit::FitFile::read(filepath);
    f.multiple_messages("Record").iter().for_each(|r| {
        println!("{:#?}", r.all_values());
    })
}
