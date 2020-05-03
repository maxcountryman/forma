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

fn main() -> io::Result<()> {
    let Opt {
        input,
        check,
        max_width,
    } = Opt::from_args();

    match input {
        Some(input) => {
            let sql_string = fs::read_to_string(&input)?;
            let formatted = format(sql_string, check, max_width)?;
            let mut writer = fs::File::create(input.clone())?;
            writer.write_all(
                &formatted
                    .iter()
                    .flat_map(|ps| ps.as_bytes().to_owned())
                    .collect::<Vec<u8>>()[..],
            )
        }
        None => {
            let mut sql_string = String::new();
            io::stdin().lock().read_to_string(&mut sql_string)?;
            let formatted = format(sql_string, check, max_width)?;
            let mut writer = io::stdout();
            writer.write_all(
                &formatted
                    .iter()
                    .flat_map(|ps| ps.as_bytes().to_owned())
                    .collect::<Vec<u8>>()[..],
            )
        }
    }
}
