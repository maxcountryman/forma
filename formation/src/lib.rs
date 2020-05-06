#![feature(box_syntax, box_patterns)]
use std::io;
use std::io::Error;

// where should this live
pub enum FormaError {
    InvalidInput, // "Unable to parse given input as SQL",
    WouldFormat,  // "Formatting would occur (happens only when check == true)"
    TransformationFailure(Error),
    Utf8Failure, // This might want to be more generic, like the stage that it fails at? render failure?
}

impl From<FormaError> for Error {
    fn from(error: FormaError) -> Self {
        match error {
            FormaError::InvalidInput => io::Error::new(io::ErrorKind::InvalidData, ""),
            FormaError::Utf8Failure => io::Error::new(io::ErrorKind::InvalidData, ""),
            FormaError::WouldFormat => io::Error::new(io::ErrorKind::InvalidData, ""),
            FormaError::TransformationFailure(err) => err
        }
    }
}

pub use crate::format::format;

mod dialect;
mod doc;
pub mod format;
