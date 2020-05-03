use std::io::{self, Write};

use sqlparser::ast::Statement;
use sqlparser::parser::Parser;

use crate::dialect::TemplatedDialect;
use crate::doc::render_statement;

fn format_statement<W: Write>(
    sql_string: String,
    statement: Statement,
    writer_callback: &Box<dyn Fn() -> io::Result<W>>,
    check: bool,
    max_width: usize,
) -> io::Result<usize> {
    let pretty = render_statement(statement, max_width)?;
    if check && pretty != sql_string {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Would format statements",
        ))
    } else {
        writer_callback()?.write(pretty.to_owned().as_bytes())
    }
}

pub fn format<W: Write>(
    sql_string: String,
    writer_callback: Box<dyn Fn() -> io::Result<W>>,
    check: bool,
    max_width: usize,
) -> io::Result<()> {
    let dialect = TemplatedDialect {};
    let statements = Parser::parse_sql(&dialect, sql_string.clone()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unable to parse given input as SQL",
        )
    })?;
    for statement in statements {
        format_statement(
            sql_string.clone(),
            statement,
            &writer_callback,
            check,
            max_width,
        )?;
    }

    Ok(())
}
