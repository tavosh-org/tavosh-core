//! Versioned terminal byte-stream fixtures. See docs/protocol.md.

use std::io::{self, Read, Write};

const MAGIC: &[u8; 8] = b"TAVOTRC\x01";
pub const MAX_PAYLOAD: usize = 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub struct Record {
    pub delta_ms: u32,
    pub payload: Vec<u8>,
}

pub struct Reader<R>(R);

impl<R: Read> Reader<R> {
    pub fn new(mut input: R) -> io::Result<Self> {
        let mut magic = [0; MAGIC.len()];
        input.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid trace signature or version",
            ));
        }
        Ok(Self(input))
    }

    pub fn read_record(&mut self) -> io::Result<Option<Record>> {
        let mut delta = [0; 4];
        match self.0.read_exact(&mut delta[..1]) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => return Err(error),
        }
        self.0.read_exact(&mut delta[1..])?;
        let mut length = [0; 4];
        self.0.read_exact(&mut length)?;
        let length = u32::from_le_bytes(length) as usize;
        if length > MAX_PAYLOAD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trace payload exceeds limit",
            ));
        }
        let mut payload = vec![0; length];
        self.0.read_exact(&mut payload)?;
        Ok(Some(Record {
            delta_ms: u32::from_le_bytes(delta),
            payload,
        }))
    }
}

pub struct Writer<W>(W);

impl<W: Write> Writer<W> {
    pub fn new(mut output: W) -> io::Result<Self> {
        output.write_all(MAGIC)?;
        Ok(Self(output))
    }

    pub fn write_record(&mut self, delta_ms: u32, payload: &[u8]) -> io::Result<()> {
        if payload.len() > MAX_PAYLOAD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trace payload exceeds limit",
            ));
        }
        self.0.write_all(&delta_ms.to_le_bytes())?;
        self.0.write_all(&(payload.len() as u32).to_le_bytes())?;
        self.0.write_all(payload)
    }

    pub fn finish(mut self) -> io::Result<W> {
        self.0.flush()?;
        Ok(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_with_fragmented_reads() {
        let mut writer = Writer::new(Vec::new()).unwrap();
        writer.write_record(0x04030201, b"\x1b[H\x00\xff").unwrap();
        writer.write_record(0, b"").unwrap();
        let bytes = writer.finish().unwrap();
        assert_eq!(&bytes[8..16], &[1, 2, 3, 4, 5, 0, 0, 0]);
        struct Fragmented<R>(R);
        impl<R: Read> Read for Fragmented<R> {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                let count = output.len().min(1);
                self.0.read(&mut output[..count])
            }
        }
        let fragmented = Fragmented(bytes.as_slice());
        let mut reader = Reader::new(fragmented).unwrap();
        assert_eq!(
            reader.read_record().unwrap(),
            Some(Record {
                delta_ms: 0x04030201,
                payload: b"\x1b[H\x00\xff".to_vec()
            })
        );
        assert_eq!(
            reader.read_record().unwrap(),
            Some(Record {
                delta_ms: 0,
                payload: vec![]
            })
        );
        assert_eq!(reader.read_record().unwrap(), None);
    }

    #[test]
    fn rejects_invalid_version_truncation_and_excess_length() {
        assert!(Reader::new(&b"TAVOTRC\x02"[..]).is_err());
        let mut writer = Writer::new(Vec::new()).unwrap();
        assert!(writer.write_record(0, &vec![0; MAX_PAYLOAD + 1]).is_err());
        writer.write_record(1, b"abc").unwrap();
        let mut bytes = writer.finish().unwrap();
        for end in 9..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]).unwrap();
            assert_eq!(
                reader.read_record().unwrap_err().kind(),
                io::ErrorKind::UnexpectedEof
            );
        }
        bytes[12..16].copy_from_slice(&(MAX_PAYLOAD as u32 + 1).to_le_bytes());
        let mut reader = Reader::new(bytes.as_slice()).unwrap();
        assert_eq!(
            reader.read_record().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
