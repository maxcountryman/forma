//! An opinionated SQL formatter
//!
//! This provides a command line utility for formatting SQL. Input may be
//! provided as either a file path or stdin. In the case of the file path, the
//! original file will be reformatted unless the `--check` flag is provided.
//! Input from stdin will be output to stdout.
//!
//! The companion library, [`formation`], is generalized to be usable in other
//! contexts.
//!
//! [`formation`]: ../formation/index.html

#![deny(clippy::all, missing_docs)]

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;

use formation::format;

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

/// Given a writer, writes the given formatted buffers.
fn write_formatted<W: Write>(mut writer: W, formatted: Vec<String>) -> io::Result<()> {
    writer.write_all(
        &formatted
            .iter()
            .flat_map(|ps| ps.as_bytes().to_owned())
            .collect::<Vec<u8>>()[..],
    )
}

fn main() -> io::Result<()> {
    let Opt {
        input,
        check,
        max_width,
    } = Opt::from_args();

    match input {
        // `PathBuf` provided, so let's use that.
        Some(input) => {
            let sql_string = fs::read_to_string(&input)?;
            let formatted = format(sql_string, check, max_width)?;
            let writer = fs::File::create(input)?;
            write_formatted(writer, formatted)
        }

        // Otherwise use stdin and stdout.
        None => {
            let mut sql_string = String::new();
            io::stdin().lock().read_to_string(&mut sql_string)?;
            let formatted = format(sql_string, check, max_width)?;
            let writer = io::stdout();
            write_formatted(writer, formatted)
        }
    }
}
