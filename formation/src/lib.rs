//! An opinionated SQL formatter.
//!
//! This provides a library which exposes a function [`format`], for
//! formatting SQL. See the companion binary [`forma`], for a command line
//! utility that utilitizes this library.
//!
//! The style of formatting is intended to be opinionated and so generally not
//! configurable with the exception of the maximum width.
//!
//! Currently `formation` uses a generic SQL dialect that understands
//! templated strings in the form of `{{...}}`. Configurable dialects may be
//! added in the future.
//!
//! [`format`]: format/fn.format.html
//! [`forma`]: ../forma/index.html

#![deny(clippy::all, missing_docs)]
#![feature(box_syntax, box_patterns)]
use std::io;

// TODO: Relocate this?
/// Forma error type.
#[derive(Debug)]
pub enum FormaError {
    /// Unable to parse given input as SQL.
    InvalidInput,
    /// Formatting would occur, i.e. when `check` is `true`.
    WouldFormat,
    /// A transformation failure that wraps `io::Error`.
    TransformationFailure(io::Error),
    /// A UTF8 failure.
    Utf8Failure,
}

impl From<FormaError> for io::Error {
    fn from(error: FormaError) -> Self {
        match error {
            FormaError::InvalidInput => io::Error::new(io::ErrorKind::InvalidData, ""),
            FormaError::Utf8Failure => io::Error::new(io::ErrorKind::InvalidData, ""),
            FormaError::WouldFormat => io::Error::new(io::ErrorKind::InvalidData, ""),
            FormaError::TransformationFailure(err) => err,
        }
    }
}

pub use crate::format::format;

mod dialect;
mod doc;
pub mod format;
