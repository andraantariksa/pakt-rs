use std::io::{Error, self};
use crate::error::ErrorKind::IOError;

#[derive(Debug)]
pub enum ErrorKind {
    IOError(io::Error),
    InvalidMagicNumber,
    InvalidVersion
}

impl From<io::Error> for ErrorKind {
    fn from(error: Error) -> Self {
        IOError(error)
    }
}
