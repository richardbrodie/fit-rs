use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("garmin_1000.fit");
    // for _ in 0..100 {
    let _ = new_fit::run(&filepath);
    // }
}
