use csv::ReaderBuilder;
use heck::{CamelCase, ShoutySnakeCase};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

type KeyValPair = (String, String);
type KeyValSet = (String, String, String, String);

const MESSAGE_TYPE_FILE: &'static str = "message_type_enum.rs";
const FIELD_TYPE_FILE: &'static str = "field_type_enum.rs";

const MATCH_MESSAGE_TYPE_FILE: &'static str = "match_message_type.rs";

const MATCH_MESSAGE_FIELD_FILE: &'static str = "match_message_field.rs";
const MATCH_MESSAGE_OFFSET_FILE: &'static str = "match_message_offset.rs";
const MATCH_MESSAGE_SCALE_FILE: &'static str = "match_message_scale.rs";

const MATCH_CUSTOM_ENUM_FILE: &'static str = "match_custom_enum.rs";

fn main() {
    let mut in_file = File::open("data/types.semi.csv").unwrap();
    let mut contents = String::new();
    in_file.read_to_string(&mut contents).unwrap();
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(contents.as_bytes());

    let mut type_name = String::new();
    let mut type_type = String::new();
    let mut type_buf: Vec<KeyValPair> = Vec::new();
    let mut types_store: HashMap<String, Vec<KeyValPair>> = HashMap::new();

    for r in rdr.records().flatten() {
        if !r[0].is_empty() {
            if !type_name.is_empty() {
                types_store.insert(type_name.clone(), type_buf);
            }
            type_name = (&r[0]).into();
            type_type = (&r[1]).into();
            type_buf = Vec::new();
        }
        if r[0].is_empty() && !r[2].is_empty() {
            if r[4].contains("Deprecated") {
                continue;
            }
            type_buf.push(((&r[3]).into(), (&r[2]).into()));
        }
    }
    types_store.insert(type_name.clone(), type_buf);

    let mut in_file = File::open("data/messages.semi.csv").unwrap();
    let mut contents = String::new();
    in_file.read_to_string(&mut contents).unwrap();
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(contents.as_bytes());

    let mut msg_name = String::new();
    let mut msg_buf: Vec<KeyValSet> = Vec::new();
    let mut msgs_store: HashMap<String, Vec<KeyValSet>> = HashMap::new();
    let mut fields_set: HashSet<String> = HashSet::new();

    for r in rdr.records().flatten() {
        if !r[0].is_empty() {
            if !msg_name.is_empty() {
                msgs_store.insert(msg_name.clone(), msg_buf);
            }
            msg_name = (&r[0]).into();
            msg_buf = Vec::new();
        }
        if r[0].is_empty() && !r[1].is_empty() && !r[2].is_empty() {
            fields_set.insert(r[3].into());
            let val = if r[2].ends_with("_lat") || r[2].ends_with("_long") {
                "Coordinates"
            } else {
                &r[3]
            };
            msg_buf.push((r[1].into(), val.into(), r[6].into(), r[7].into()));
        }
    }
    msgs_store.insert(msg_name.clone(), msg_buf);

    let msg_types = types_store.get("mesg_num").unwrap();
    let msgs: HashSet<_> = msgs_store.keys().collect();

    write_field_type_enum(&fields_set);
    write_match_message_field(&msgs, &msgs_store);
    write_match_message_offset(&msgs, &msgs_store);
    write_match_message_scale(&msgs, &msgs_store);
    write_match_message_type(&msg_types);
    write_message_type_enum(&msgs);
    write_custom_type_match(&fields_set, &types_store);
}

fn write_custom_type_match(set: &HashSet<String>, types_store: &HashMap<String, Vec<KeyValPair>>) {
    // fn match_custom_type(FieldType, u8) -> Option<&'static str>;
    let mut outfile = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join(MATCH_CUSTOM_ENUM_FILE))
            .unwrap(),
    );
    writeln!(
        &mut outfile,
        "pub fn enum_type(f: FieldType, k: u16) -> Option<&'static str> {{\n    match f {{",
    )
    .unwrap();
    // for v in set.difference(&primitive_set()) {
    for f in set {
        if let Some(map) = types_store.get(f) {
            // dbg!(map);
            writeln!(
                &mut outfile,
                "        FieldType::{} => match k {{",
                f.to_camel_case(),
            )
            .unwrap();
            for (k, e) in map.iter() {
                if let Some(k) = parse_u16(k) {
                    writeln!(&mut outfile, "            {} => Some({:?}),", k, e).unwrap();
                }
            }
            writeln!(&mut outfile, "            _ => None,").unwrap();
            writeln!(&mut outfile, "         }}").unwrap();
        } else {
            continue;
        }
    }
    writeln!(&mut outfile, "        FieldType::None => None,",).unwrap();
    writeln!(&mut outfile, "        _ => None\n    }}\n}}",).unwrap();
}

// fn write_custom_type_funs(types_store: &HashMap<TypeKey, HashMap<String, String>>) -> String {
// // fn my_custom_type(u8) -> Option<&'static str>;
// let mut s = String::new();
// for (k, v) in types_store {
// let (n, t) = k;
// let rt = map_primitive(&t);
// s = format!(
// "{}pub fn {}(k: {}) -> Option<&'static str> {{\n    match k {{\n",
// s, n, rt
// );
// for (a, b) in v {
// writeln!(&mut outfile, "        {} => Some({:?}),\n", s, a, b);
// }
// writeln!(&mut outfile, "        _ => None\n    }}\n}}\n", s);
// }
// s
// }

fn write_match_message_field(set: &HashSet<&String>, msgs_store: &HashMap<String, Vec<KeyValSet>>) {
    // fn match_field_type(MessageType, u16) -> FieldType;
    let mut outfile = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join(MATCH_MESSAGE_FIELD_FILE))
            .unwrap(),
    );
    for v in set {
        let map = msgs_store.get(v.to_owned()).unwrap();
        let len = map
            .iter()
            .map(|(x, _, _, _)| x.parse::<u16>().unwrap())
            .max()
            .unwrap()
            + 1;
        let mut s = format!(
            "static F_{}: [FieldType; {}] = [",
            v.to_shouty_snake_case(),
            len
        );
        for k in 0..len {
            let i = format!("{}", k);
            if let Some((_, v, _, _)) = map.iter().find(|a| a.0 == i) {
                s = format!("{}FieldType::{},", s, v.to_camel_case());
            } else {
                s.push_str("FieldType::None,");
            }
        }
        s.push_str("];");
        writeln!(&mut outfile, "{}", s).unwrap();
    }
    writeln!(
        &mut outfile,
        "pub fn match_message_field(m: MessageType) -> &'static [FieldType] {{\n    match m {{",
    )
    .unwrap();
    for v in set {
        writeln!(
            &mut outfile,
            "        MessageType::{} => &F_{},",
            v.to_camel_case(),
            v.to_shouty_snake_case()
        )
        .unwrap();
    }
    writeln!(
        &mut outfile,
        "        MessageType::None => panic!(\"cannot call this with a None variant\"),",
    )
    .unwrap();
    writeln!(
        &mut outfile,
        "        _ => panic!(\"invalid variant used\")\n    }}\n}}",
    )
    .unwrap();
}

fn write_match_message_offset(
    set: &HashSet<&String>,
    msgs_store: &HashMap<String, Vec<KeyValSet>>,
) {
    // fn match_field_type(MessageType, u16) -> FieldType;
    let mut outfile = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join(MATCH_MESSAGE_OFFSET_FILE))
            .unwrap(),
    );
    for v in set {
        let map = msgs_store.get(v.to_owned()).unwrap();
        let len = map
            .iter()
            .map(|(x, _, _, _)| x.parse::<u16>().unwrap())
            .max()
            .unwrap()
            + 1;
        let mut s = format!(
            "static OS_{}: [Option<i16>; {}] = [",
            v.to_shouty_snake_case(),
            len
        );
        for k in 0..len {
            let i = format!("{}", k);
            if let Some((_, _, _, vv)) = map.iter().find(|a| a.0 == i && !a.3.is_empty()) {
                s = format!("{}Some({}i16),", s, vv);
            } else {
                s.push_str("None,");
            }
        }
        s.push_str("];");
        writeln!(&mut outfile, "{}", s).unwrap();
    }
    writeln!(
        &mut outfile,
        "pub fn match_message_offset(m: MessageType) -> &'static [Option<i16>] {{\n    match m {{",
    )
    .unwrap();
    for v in set {
        writeln!(
            &mut outfile,
            "        MessageType::{} => &OS_{},",
            v.to_camel_case(),
            v.to_shouty_snake_case()
        )
        .unwrap();
    }
    writeln!(
        &mut outfile,
        "        MessageType::None => panic!(\"cannot call this with a None variant\"),",
    )
    .unwrap();
    writeln!(
        &mut outfile,
        "        _ => panic!(\"invalid variant used\")\n    }}\n}}",
    )
    .unwrap();
}

fn write_match_message_scale(set: &HashSet<&String>, msgs_store: &HashMap<String, Vec<KeyValSet>>) {
    // fn match_field_type(MessageType, u16) -> FieldType;
    let mut outfile = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join(MATCH_MESSAGE_SCALE_FILE))
            .unwrap(),
    );
    for v in set {
        let map = msgs_store.get(v.to_owned()).unwrap();
        let len = map
            .iter()
            .map(|(x, _, _, _)| x.parse::<u16>().unwrap())
            .max()
            .unwrap()
            + 1;
        let mut s = format!(
            "static S_{}: [Option<f32>; {}] = [",
            v.to_shouty_snake_case(),
            len
        );
        for k in 0..len {
            let i = format!("{}", k);
            if let Some((_, _, vv, _)) = map.iter().find(|a| a.0 == i && !a.2.is_empty()) {
                let vvv = vv.split(',').nth(0).unwrap();
                s = format!("{}Some({}f32),", s, vvv);
            } else {
                s.push_str("None,");
            }
        }
        s.push_str("];");
        writeln!(&mut outfile, "{}", s).unwrap();
    }
    writeln!(
        &mut outfile,
        "pub fn match_message_scale(m: MessageType) -> &'static [Option<f32>] {{\n    match m {{",
    )
    .unwrap();
    for v in set {
        writeln!(
            &mut outfile,
            "        MessageType::{} => &S_{},",
            v.to_camel_case(),
            v.to_shouty_snake_case()
        )
        .unwrap();
    }
    writeln!(
        &mut outfile,
        "        MessageType::None => panic!(\"cannot call this with a None variant\"),",
    )
    .unwrap();
    writeln!(
        &mut outfile,
        "        _ => panic!(\"invalid variant used\")\n    }}\n}}",
    )
    .unwrap();
}

fn write_match_message_type(map: &Vec<KeyValPair>) {
    // fn match_message_type(u16) -> MessageType;
    let mut outfile = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join(MATCH_MESSAGE_TYPE_FILE))
            .unwrap(),
    );
    writeln!(
        &mut outfile,
        "pub fn match_message_type(k: u16) -> MessageType {{\n    match k {{"
    )
    .unwrap();
    for (k, v) in map {
        writeln!(
            &mut outfile,
            "        {} => MessageType::{},",
            k,
            v.to_camel_case()
        )
        .unwrap();
    }
    writeln!(&mut outfile, "        _ => MessageType::None\n    }}\n}}").unwrap();
}

fn write_field_type_enum(set: &HashSet<String>) {
    // enum FieldType;
    let mut outfile = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join(FIELD_TYPE_FILE)).unwrap(),
    );
    writeln!(
        &mut outfile,
        "#[derive(Debug, Copy, Clone, PartialEq)]\npub enum FieldType {{"
    )
    .unwrap();
    for v in set {
        writeln!(&mut outfile, "    {},", v.to_camel_case()).unwrap();
    }
    writeln!(&mut outfile, "    Coordinates,").unwrap();
    writeln!(&mut outfile, "    None,\n}}").unwrap();
}

fn write_message_type_enum(set: &HashSet<&String>) {
    // enum MessageType;
    let mut outfile = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join(MESSAGE_TYPE_FILE)).unwrap(),
    );
    writeln!(
        &mut outfile,
        "#[derive(Debug, Copy, Clone, PartialEq)]\npub enum MessageType {{"
    )
    .unwrap();
    for v in set {
        writeln!(&mut outfile, "    {},", v.to_camel_case()).unwrap();
    }
    writeln!(&mut outfile, "    Pad,").unwrap();
    writeln!(&mut outfile, "    MfgRangeMax,").unwrap();
    writeln!(&mut outfile, "    MfgRangeMin,").unwrap();
    writeln!(&mut outfile, "    None,\n}}").unwrap();
}

// fn map_primitive(p: &str) -> &str {
// match p {
// "enum" | "uint8" | "uint8z" => "u8",
// "uint16" | "uint16z" => "u16",
// "uint32" | "uint32z" => "u32",
// _ => panic!("unrecognised type: '{:?}'", p),
// }
// }
// fn primitive_set() -> HashSet<String> {
// use std::iter::FromIterator;
// const primitives: [&'static str; 18] = [
// "bool", "enum", "sint8", "uint8", "sint16", "uint16", "sint32", "uint32", "string",
// "float32", "float64", "uint8z", "uint16z", "uint32z", "byte", "sint64", "uint64",
// "uint64z",
// ];
// HashSet::from_iter(primitives.iter().map(|x| x.to_string()))
// }
fn parse_u16(s: &str) -> Option<u16> {
    const HEX: &'static str = "0x";
    let l = s.to_lowercase();
    if l.starts_with(HEX) {
        u16::from_str_radix(&l.trim_start_matches(HEX), 16).ok()
    } else {
        u16::from_str_radix(&l, 10).ok()
    }
}
