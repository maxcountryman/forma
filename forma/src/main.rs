//! An opinionated SQL formatter
//!
//! This provides a command line utility for formatting SQL. Input may be provided as either a file
//! path or stdin. In the case of the former, the original file will be re-formatted unless the
//! `--check` flag is provided. (The `--check` flag will return a non-zero error code if `forma`
//! would reformat the given input.) Input from stdin will be output to stdout.
//!
//! The companion library [`formation`], is generalized to be usable in other contexts, such as
//! your own programs.
//!
//! [`formation`]: ../formation/index.html

#![deny(clippy::all, missing_docs)]
#![feature(with_options)]

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::Result;
use formation::format;
use structopt::StructOpt;

const DEFAULT_MAX_WIDTH: &str = "100";

#[derive(StructOpt)]
#[structopt(name = "forma", about = "🐚 An opinionated SQL formatter.")]
struct Opt {
    /// A SQL input to format; either a file path or stdin.
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Check if formatting would occur without applying it.
    #[structopt(long)]
    check: bool,

    /// The maximum allowed column width before wrapping.
    #[structopt(long = "max-width", default_value = DEFAULT_MAX_WIDTH)]
    max_width: usize,
}

/// Given a reader, a writer, a check bool, and max width to format to, formats the reader's string
/// value and then writes the result via the writer.
fn formatter<R, W>(mut reader: R, mut writer: W, check: bool, max_width: usize) -> Result<()>
where
    W: Write,
    R: BufRead,
{
    let mut sql_string = String::new();
    reader.read_to_string(&mut sql_string)?;
    let formatted = format(&sql_string, check, max_width)?;
    writer.write_all(
        &formatted
            .iter()
            .flat_map(|ps| ps.as_bytes().to_owned())
            .collect::<Vec<u8>>()[..],
    )?;
    Ok(())
}

/// Main entrypoint for the `forma` binary.
fn main() -> Result<()> {
    let Opt {
        input,
        check,
        max_width,
    } = Opt::from_args();
    match input {
        Some(input) => formatter(
            BufReader::new(fs::File::open(&input)?),
            fs::File::with_options().write(true).open(input)?,
            check,
            max_width,
        ),
        None => formatter(io::stdin().lock(), io::stdout(), check, max_width),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter() -> Result<()> {
        let input = b"SELECT * FROM t1";
        let mut output = Vec::new();
        formatter(&input[..], &mut output, false, 100)?;
        let output = String::from_utf8(output)?;
        assert_eq!(output, "select * from t1;\n");
        Ok(())
    }
}
