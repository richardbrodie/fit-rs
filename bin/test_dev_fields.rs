use std::path::PathBuf;

fn main() {
    let filepath = PathBuf::from("data/working_wahoo_dev_fields.fit_");
    fit::run(&filepath);
}
