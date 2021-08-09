mod error;
mod util;

use std::io;
use std::hash::Hasher;
use std::io::{Read, Seek, SeekFrom};

const MAGIC: u32 = 0x43504448;
const VERSION: u32 = 0x00000001;

struct Pak {

}

impl Pak {
    fn from(mut buffer: impl Read) -> Result<Self, error::ErrorKind> {
        let mut temp_buffer_4_bytes = [0u8; 4];
        buffer.read(&mut temp_buffer_4_bytes)?;


        Ok(Self {})
    }
}

trait SeekRead: Seek + Read {}
impl<T: Seek + Read> SeekRead for T {}

struct Content {
    pub name: String,
    pub file: Box<dyn SeekRead>
}

pub struct Encoder {
    contents: Vec<Content>
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            contents: vec![]
        }
    }

    pub fn add_file(&mut self, path: &str, file: Box<dyn SeekRead>) {
        self.contents.push(Content {
            file,
            name: path.to_owned()
        });
    }

    pub fn write(&mut self, mut buffer: impl io::Write) {
        // Magic number
        buffer.write(&MAGIC.to_le_bytes());
        // Version
        buffer.write(&VERSION.to_le_bytes());
        // Reserved
        buffer.write(&[0u8; 20 * 4]);
        buffer.write(&(self.contents.len() as u32).to_le_bytes());

        for content in self.contents.iter_mut() {
            // Name
            buffer.write(content.name.as_bytes());
            // TODO Offset
            buffer.write(content.name.as_bytes());
            // Size
            buffer.write(&(content.file.seek(SeekFrom::End(0)).unwrap() as u32).to_le_bytes());
        }
        for content in self.contents.iter_mut() {
            io::copy(&mut content.file, &mut buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
