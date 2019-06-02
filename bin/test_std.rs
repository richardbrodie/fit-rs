use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("data/garmin_1000.fit");
    fit::run(&filepath);
}
