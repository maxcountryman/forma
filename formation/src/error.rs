//! Error module
//!
//! Provides a custom error enum representing different errors the formatter can encounter.
use std::io;

/// An alias for a `std::result::Result` that pins `FormaError`.
pub type Result<T> = std::result::Result<T, FormaError>;

/// Forma error type.
#[derive(Debug)]
pub enum FormaError {
    /// Unable to parse given input as SQL.
    InvalidInput,
    /// Formatting would occur, i.e. when `check` is `true`.
    WouldFormat,
    /// A transformation failure that wraps `io::Error`.
    TransformationFailure(io::Error),
    /// A UTF-8 failure.
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
