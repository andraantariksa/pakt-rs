use std::io;
use std::io::Error;
use crate::error::ErrorKind::IOError;

pub enum ErrorKind {
    IOError(io::Error)
}

impl From<io::Error> for ErrorKind {
    fn from(error: Error) -> Self {
        IOError(error)
    }
}
