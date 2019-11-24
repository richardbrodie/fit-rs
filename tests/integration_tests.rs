use std::path::PathBuf;

#[test]
fn it_reads_garmin_1000() {
    let filepath = PathBuf::from("data/garmin_1000.fit");
    read_file(filepath);
}

#[test]
fn it_reads_garmin_520_long() {
    let filepath = PathBuf::from("data/garmin_520_long.fit");
    read_file(filepath);
}

#[test]
fn it_reads_2019_garmin_520() {
    let filepath = PathBuf::from("data/2019_garmin_520.fit");
    read_file(filepath);
}

#[test]
fn it_reads_wahoo_elemnt() {
    let filepath = PathBuf::from("data/wahoo_elemnt.fit");
    read_file(filepath);
}

#[test]
fn it_reads_wahoo_elemnt_dev_fields() {
    let filepath = PathBuf::from("data/wahoo_elemnt_dev_fields.fit");
    read_file(filepath);
}

#[test]
fn it_reads_wahoo_kickr() {
    let filepath = PathBuf::from("data/wahoo_kickr.fit");
    read_file(filepath);
}

#[test]
fn it_reads_tacx_flux() {
    let filepath = PathBuf::from("data/tacx_flux.fit");
    read_file(filepath);
}

#[test]
fn it_reads_all_files() {
    for entry in std::fs::read_dir("data").unwrap() {
        let path = entry.unwrap().path();
        if let Some(s) = &path.extension() {
            if let Some("fit") = s.to_str() {
                read_file(path);
            }
        }
    }
}



fn read_file(f: PathBuf) {
    let f = fit::Fit::new(&f);
    for m in f {
        m.kind;
    }
}
