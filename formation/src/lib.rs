#![feature(box_syntax, box_patterns)]

pub use crate::dialect::TemplatedDialect;
pub use crate::doc::prettify_statement;

pub mod dialect;
pub mod doc;
