use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("data/working_wahoo_dev_fields.fit_");
    let f = fit::Fit::new(&filepath);
    for m in f {
        m.kind;
    }

}
