use fit::Value;
use std::io::{BufReader, Error, Read, Seek, SeekFrom, Take};

use super::definition_record::{BaseType, DefinitionRecord, FieldDefinition};
use crate::consts::*;
use crate::reader::{Endian, Reader};

#[derive(Debug)]
pub struct DataRecord {
    pub global_message_num: u16,
    pub fields: Vec<DataField>,
}
impl DataRecord {
    pub fn new(reader: &mut Reader, definition: &DefinitionRecord) -> Self {
        let fields = &definition
            .field_defs
            .map(|fd| DataField::new(reader, &definition.architecture, &fd))
            .collect();
        Self {
            global_message_num: definition.global_message_num,
            fields: fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn it_reads_a_data_record() {
        let mut reader = fit_setup();
        reader.skip(14); // FileHeader
        reader.skip(1); // HeaderByte
        let definition = DefinitionRecord::new(&mut reader, false);
        reader.skip(1); // HeaderByte
        let data = DataRecord::new(&mut reader, &definition);
        assert_eq!(data.fields[0].values[0], Value::U32(3902378567)); // base type 12
        assert_eq!(data.fields[1].values[0], Value::U32(849790468));
        assert_eq!(data.fields[3].values[0], Value::U16(1));
        assert_eq!(
            fit::get_message_struct(&definition.global_message_num)
                .unwrap()
                .msg_name(),
            "File Id"
        );
    }
}
