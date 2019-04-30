use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("garmin_1000.fit");
    // for _ in 0..100 {
    new_fit::run(&filepath);
    // }
}
