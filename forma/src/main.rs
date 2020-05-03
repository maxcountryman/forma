use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::exit;

use sqlparser::parser::Parser;
use structopt::StructOpt;

use formation::{prettify_statement, TemplatedDialect};

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

fn format<W: Write>(
    input_string: String,
    mut writer: W,
    check: bool,
    max_width: usize,
) -> io::Result<()> {
    let dialect = TemplatedDialect {};
    let statements = Parser::parse_sql(&dialect, input_string.clone()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unable to parse given input as SQL",
        )
    })?;

    for statement in statements {
        let pretty = prettify_statement(statement, max_width)?;
        if check && pretty != input_string {
            writer.write(input_string.as_bytes())?;
            exit(1)
        } else {
            writer.write(pretty.to_owned().as_bytes())?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let Opt {
        input,
        check,
        max_width,
    } = Opt::from_args();

    match input {
        Some(input) => {
            let input_string = fs::read_to_string(&input)?;
            format(input_string, fs::File::create(input)?, check, max_width)
        }
        None => {
            let mut input_string = String::new();
            io::stdin().lock().read_to_string(&mut input_string)?;
            format(input_string, io::stdout(), check, max_width)
        }
    }
}
