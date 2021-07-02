use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("data/garmin_1000.fit");
    for _ in 1..100 {
        let f = fit::Fit::new(&filepath);
        for m in f {
            m.kind;
        }
    }
}
