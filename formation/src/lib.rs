//! An opinionated SQL formatter
//!
//! This provides a library which exposes a function [`format`], for formatting SQL. See the
//! companion binary [`forma`], for a command-line utility that consumes this library.
//!
//! The style of formatting is intended to be opinionated and so generally not configurable. The
//! primary exception is the ability to set a maximum width which the formatter will attempt to
//! adhere the output to.
//!
//! Currently `formation` uses a generic SQL dialect that understands templated strings in the form
//! of `{{ .. }}`. Configurable dialects may be added in the future.
//!
//! [`format`]: format/fn.format.html
//! [`forma`]: ../forma/index.html

#![deny(clippy::all, missing_docs)]
#![feature(box_syntax, box_patterns)]

mod dialect;
mod doc;
pub mod error;
pub mod format;

pub use crate::format::format;
