use std::fs;
use std::io::{self, Read};
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
            let input_string = fs::read_to_string(&input)?;
            format(
                input_string,
                Box::new(move || fs::File::create(input.clone())),
                check,
                max_width,
            )
        }
        None => {
            let mut input_string = String::new();
            io::stdin().lock().read_to_string(&mut input_string)?;
            format(
                input_string,
                Box::new(|| Ok(io::stdout())),
                check,
                max_width,
            )
        }
    }
}
