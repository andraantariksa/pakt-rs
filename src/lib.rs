mod error;
mod util;

use std::io::{self, Read, Seek, SeekFrom, BufRead, Write};
use std::collections::HashMap;
use crate::error::ErrorKind;
use crate::util::as_u32_le;

const MAGIC: &str = "PAKT";
const VERSION: u32 = 1;

struct FileInfo {
    offset: u32,
    size: u32
}

struct Decoder<'a> {
    buffer: Box<dyn SeekRead + 'a>,
    total_files: u32,
    contents: HashMap<String, FileInfo>
}

impl<'a> Decoder<'a> {
    fn from(mut buffer: impl SeekRead + 'a) -> Result<Self, error::ErrorKind> {
        let mut temp_buffer_4_bytes = [0u8; 4];
        buffer.read(&mut temp_buffer_4_bytes)?;
        if temp_buffer_4_bytes != MAGIC.as_bytes() {
            return Err(ErrorKind::InvalidMagicNumber);
        }
        buffer.read(&mut temp_buffer_4_bytes)?;
        if temp_buffer_4_bytes != VERSION.to_le_bytes() {
            return Err(ErrorKind::InvalidVersion);
        }
        buffer.seek(SeekFrom::Current(20 * 4));
        buffer.read(&mut temp_buffer_4_bytes)?;
        let total_files = as_u32_le(&temp_buffer_4_bytes);

        let mut contents = HashMap::new();
        let mut string_buf = Vec::<u8>::new();
        for _ in 0..total_files {
            string_buf.clear();
            // Name
            buffer.read(&mut temp_buffer_4_bytes)?; // String length
            string_buf.resize(as_u32_le(&temp_buffer_4_bytes) as usize, 0);
            buffer.read(&mut string_buf)?;
            // Offset
            buffer.read(&mut temp_buffer_4_bytes)?;
            let offset = as_u32_le(&temp_buffer_4_bytes);
            // Size
            buffer.read(&mut temp_buffer_4_bytes)?;
            let size = as_u32_le(&temp_buffer_4_bytes);

            contents.insert(String::from_utf8(string_buf.clone()).unwrap(), FileInfo {
                size,
                offset,
            });
        }

        Ok(Self {
            total_files,
            contents,
            buffer: Box::new(buffer)
        })
    }

    fn total_files(&self) -> u32 {
        self.total_files
    }

    fn extract(&mut self, path: &str, mut write: impl Write) -> Result<(), ErrorKind> {
        let file_info = &self.contents[path];
        self.buffer.seek(SeekFrom::Start(file_info.offset as u64));
        io::copy(&mut self.buffer.as_mut().take(file_info.size as u64), &mut write);
        Ok(())
    }
}

trait SeekRead: Seek + Read {}
impl<T: Seek + Read> SeekRead for T {}

pub struct BufferInfo {
    buffer: Box<dyn SeekRead>,
    size: u32
}

pub struct Encoder {
    contents: HashMap<String, BufferInfo>,
    paths_len: u32
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
            paths_len: 0
        }
    }

    pub fn add_file(&mut self, path: &str, mut buffer: Box<dyn SeekRead>) {
        self.paths_len += path.len() as u32;
        let size = buffer.seek(SeekFrom::End(0)).unwrap() as u32;
        buffer.seek(SeekFrom::Start(0));
        self.contents.insert(path.to_owned(), BufferInfo {
            buffer,
            size
        });
        println!("path {} size {}", path, size);
    }

    pub fn write(&mut self, mut write_buffer: impl io::Write) {
        // Magic number
        write_buffer.write(MAGIC.as_bytes());
        // Version
        write_buffer.write(&VERSION.to_le_bytes());
        // Reserved
        write_buffer.write(&[0u8; 20 * 4]);
        write_buffer.write(&(self.contents.len() as u32).to_le_bytes());

        let total_files = self.contents.len() as u32;
        let mut pad = 4 + 4 + 20 * 4 + 4 + self.paths_len + (4 + 4 + 4) * total_files;
        for (path, buffer_info) in self.contents.iter_mut() {
            // Name
            write_buffer.write(&(path.len() as u32).to_le_bytes()); // String length
            write_buffer.write(path.as_bytes());
            // Offset
            write_buffer.write(&pad.to_le_bytes());
            // Size
            write_buffer.write(&(buffer_info.size).to_le_bytes());
            pad += buffer_info.size;
        }
        for (path, mut read_buffer) in self.contents.iter_mut() {
            io::copy(&mut read_buffer.buffer, &mut write_buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode() {
        use super::*;
        use std::fs::File;
        use std::io::prelude::*;

        let mut encoder = Encoder::new();
        encoder.add_file("tempeh.jpg", Box::new(File::open("assets/image/tempeh.jpg").unwrap()));
        encoder.write(File::create("assets/output/encode.pakt").unwrap());
    }

    #[test]
    fn encode_and_extract() {
        use super::*;
        use std::fs::File;
        use std::io::prelude::*;
        use file_diff::diff;

        let mut encoder = Encoder::new();
        encoder.add_file("tempeh.jpg", Box::new(File::open("assets/image/tempeh.jpg").unwrap()));
        encoder.add_file("imperial.ogg", Box::new(File::open("assets/music/imperial.ogg").unwrap()));
        encoder.write(File::create("assets/output/encode_and_read.pakt").unwrap());

        let mut decoder = Decoder::from(File::open("assets/output/encode_and_read.pakt").unwrap()).unwrap();
        assert_eq!(decoder.total_files(), 2);
        decoder.extract("tempeh.jpg", File::create("assets/output/tempeh.jpg").unwrap());
        decoder.extract("imperial.ogg", File::create("assets/output/imperial.ogg").unwrap());
        assert!(diff("assets/output/tempeh.jpg", "assets/image/tempeh.jpg"));
        assert!(diff("assets/output/imperial.ogg", "assets/music/imperial.ogg"));
    }
}
