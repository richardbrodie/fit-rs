use csv::ReaderBuilder;
use heck::{CamelCase, TitleCase};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

pub struct MessageField {
    pub name: String,
    pub kind: String,
    pub scale: f32,
    pub offset: f32,
}

fn main() {
    read_types_csv();
    read_messages_csv();
}

fn read_types_csv() {
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("types.rs");
    let mut out_file = BufWriter::new(File::create(&out_path).unwrap());

    let mut in_file = File::open("types.semi.csv").unwrap();
    let mut contents = String::new();
    in_file.read_to_string(&mut contents).unwrap();

    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(contents.as_bytes());

    let mut subsequent = false;
    for r in rdr.records().flatten() {
        if !&r[0].is_empty() {
            let name = &r[0];
            if subsequent {
                write!(&mut out_file, "    _ => None\n  }}\n}}\n\n").unwrap();
            } else {
                subsequent = true;
            }
            write!(
                &mut out_file,
                "{}",
                format!(
                    "pub fn {}(key: &u32) -> Option<&'static str> {{\n  match key {{\n",
                    name
                )
            )
            .unwrap();
        }
        if !&r[3].is_empty() {
            let value = &r[2];
            if r[4].contains("Deprecated") {
                continue;
            }
            match parse_u32(&r[3]) {
                Some(key) => {
                    write!(
                        &mut out_file,
                        "{}",
                        format!("    {} => Some({:?}),\n", key, value)
                    )
                    .unwrap();
                }
                None => (),
            }
        }
    }
    write!(&mut out_file, "    _ => None\n  }}\n}}\n\n").unwrap();
}

fn read_messages_csv() {
    let def_path = Path::new(&env::var("OUT_DIR").unwrap()).join("message_definitions.rs");
    let mut def_file = BufWriter::new(File::create(&def_path).unwrap());
    let msg_path = Path::new(&env::var("OUT_DIR").unwrap()).join("messages.rs");
    let mut msg_file = BufWriter::new(File::create(&msg_path).unwrap());

    let mut in_file = File::open("messages.semi.csv").unwrap();
    let mut contents = String::new();
    in_file.read_to_string(&mut contents).unwrap();

    write!(
        &mut msg_file,
        "fn message(msg: &str) -> Option<Box<dyn DefinedMessageType>> {{\n
            match msg {{\n"
    )
    .unwrap();

    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(contents.as_bytes());

    let mut subsequent = false;
    for r in rdr.records().flatten() {
        if !&r[0].is_empty() {
            let name = &r[0];
            if subsequent {
                write!(
                    &mut def_file,
                    "             _ => None,\n        }}\n    }}\n}}\n"
                )
                .unwrap();
            } else {
                subsequent = true;
            }
            write!(
                &mut def_file,
                "{}",
                format!(
                    r#"#[derive(Debug)]
pub struct {0} {{ values: HashMap<u16, Value> }}
impl DefinedMessageType for {0} {{
    fn new() -> Self {{
        Self {{ values: HashMap::new() }} 
    }}
    fn inner(&self) -> &HashMap<u16, Value> {{
        &self.values
    }}
    fn name(&self) -> &str {{
        "{1}"
    }}
    fn write_value(&mut self, num: u16, val: Value) {{
        self.values.insert(num, val);
    }}
    fn read_value(&self, num: u16) -> Option<&Value> {{
        self.values.get(&num)
    }}
    fn defined_message_field(&self, num: u16) -> Option<&DefinedMessageField> {{
        match num {{
"#,
                    name.to_camel_case(),
                    name.to_title_case()
                )
            )
            .unwrap();
            write!(
                &mut msg_file,
                "{}",
                format!(
                    "{:?} => Some(Box::new({}::new())),\n",
                    name,
                    name.to_camel_case()
                )
            )
            .unwrap();
        }
        if !&r[1].is_empty() {
            let value = &r[2];
            if value.contains("Deprecated") {
                continue;
            }
            match parse_u32(&r[1]) {
                Some(key) => write!(
                    &mut def_file,
                    "{}",
                    format!(
                        r#"            {0} => {{
                static F: DefinedMessageField = DefinedMessageField {{
                    num: {0},
                    name: "{1}",
                    kind: "{2}",
                    scale: {3:?},
                    offset: {4:?},
                }};
                Some(&F)
            }},
"#,
                        key,
                        value,
                        &r[3],
                        &r[6].parse::<f64>().ok(),
                        &r[7].parse::<f64>().ok(),
                    )
                )
                .unwrap(),
                None => (),
            }
        }
    }
    write!(
        &mut def_file,
        "            _ => None,\n        }}\n    }}\n}}\n"
    )
    .unwrap();
    write!(
        &mut msg_file,
        "            _ => None,\n        }}\n    }}\n"
    )
    .unwrap();
}

fn parse_u32(s: &str) -> Option<u32> {
    const HEX: &'static str = "0x";
    let l = s.to_lowercase();
    if l.starts_with(HEX) {
        u32::from_str_radix(&l.trim_start_matches(HEX), 16).ok()
    } else {
        u32::from_str_radix(&l, 10).ok()
    }
}
