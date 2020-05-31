//! Error module
//!
//! Provides a custom error enum representing different errors the formatter can encounter.
use std::io;
use std::string::FromUtf8Error;

use sqlparser::parser::ParserError;
use thiserror::Error;

/// An alias for a `std::result::Result` that pins `FormaError`.
pub type Result<T> = std::result::Result<T, FormaError>;

/// Forma error type.
#[derive(Error, Debug)]
pub enum FormaError {
    /// Unable to parse given input as SQL.
    #[error("Invalid SQL provided as input")]
    InvalidInput(#[from] ParserError),

    /// Formatting would occur, i.e. when `check` is `true`.
    #[error("Check failed; would format SQL")]
    WouldFormat,

    /// A transformation failure that wraps `io::Error`.
    #[error("Transformation did not succeed")]
    TransformationFailure(#[from] io::Error),

    /// A UTF-8 failure.
    #[error("A UTF8 error occurred")]
    Utf8Failure(#[from] FromUtf8Error),
}
