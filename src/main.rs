use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("data/garmin_1000.fit");
    // let filepath = PathBuf::from("garmin_520_long.fit");

    // for _ in 0..50 {
    fit::run(&filepath);
    // }
}
