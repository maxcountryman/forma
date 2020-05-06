#![feature(box_syntax, box_patterns)]
use std::io::Error;
// where should this live
pub enum FormaError {
    InvalidInput, // "Unable to parse given input as SQL",
    WouldFormat,  // "Formatting would occur (happens only when check == true)"
    TransformationFailure(Error),
    Utf8Failure, // This might want to be more generic, like the stage that it fails at? render failure?
}

pub use crate::format::format;

mod dialect;
mod doc;
pub mod format;
