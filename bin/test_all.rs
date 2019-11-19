fn main() {
    for entry in std::fs::read_dir("data").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(s) = &path.extension() {
            if let Some("fit") = s.to_str() {
                let f = fit::Fit::new(&path);
                for m in f {
                    m.kind;
                }
            }
        }
    }
}
