use crate::reader::{Endian, Reader};
use crate::Error;

#[derive(Debug)]
pub struct FileHeader {
    filesize: u8,
    protocol: u8,
    profile_version: u16,
    pub num_record_bytes: u32,
    fileext: bool,
    crc: u16,
}

// needs 14 bytes
impl FileHeader {
    pub fn new(reader: &mut Reader) -> Result<FileHeader, Error> {
        let endianness = Endian::Little;
        Ok(FileHeader {
            filesize: reader.byte()?,
            protocol: reader.byte()?,
            profile_version: reader.u16(&endianness)?,
            num_record_bytes: reader.u32(&endianness)?,
            fileext: read_fit_string(reader.bytes(4)?.as_slice()),
            crc: reader.u16(&endianness)?,
        })
    }
    pub fn file_length(&self) -> u32 {
        self.num_record_bytes + 14
    }
}

fn read_fit_string(buffer: &[u8]) -> bool {
    let ext = ".FIT";
    buffer == ext.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::FileHeader;
    use crate::tests::fit_setup;

    #[test]
    fn it_reads_fileheader() {
        let mut reader = fit_setup().unwrap();
        let fileheader = FileHeader::new(&mut reader).unwrap();
        assert_eq!(fileheader.num_record_bytes, 191_877);
    }
}
