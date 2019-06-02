use std::path::PathBuf;

fn main() {
    for entry in std::fs::read_dir("data").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(s) = &path.extension() {
            match s.to_str() {
                Some("fit") => {
                    dbg!(&path);
                    fit::run(&path);
                }
                _ => ()
            }
        }
    }
}
