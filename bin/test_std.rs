use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("data/garmin_1000.fit");
    let f = fit::Fit::new(&filepath);
    for m in f {
        m.kind;
    }
    // dbg!(f.next());
}
