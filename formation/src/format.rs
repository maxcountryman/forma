//! The primary formatting interface.

use sqlparser::ast::Statement;
use sqlparser::parser::Parser;

use crate::dialect::TemplatedDialect;
use crate::doc::render_statement;
use crate::error::{self, FormaError};

fn format_statement(
    sql_string: String,
    statement: Statement,
    check: bool,
    max_width: usize,
) -> error::Result<String> {
    let pretty = render_statement(statement, max_width)?;
    if check && pretty != sql_string {
        Err(FormaError::WouldFormat)
    } else {
        Ok(pretty.to_string())
    }
}

/// Formats a given SQL string in accordance with the given maximum width.
///
/// # Errors
///
/// Returns a `FormaError::InvalidInput` if the parser cannot parse the
/// provided input.
///
/// If `check` is `true`, will return a `FormaError::WouldFormat` if the
/// provided input would be formatted.
///
/// # Example
///
/// ```
/// use formation::format;
/// let sql_string = "SELECT * FROM users;".to_owned();
/// assert_eq!(
///     format(sql_string, false, 100).unwrap(),
///     vec!["select\n  *\nfrom\n  users".to_owned()]
/// );
/// ```
pub fn format(sql_string: String, check: bool, max_width: usize) -> error::Result<Vec<String>> {
    let dialect = TemplatedDialect {};
    let statements =
        Parser::parse_sql(&dialect, sql_string.clone()).map_err(|_| FormaError::InvalidInput)?;
    let mut pretty_statements: Vec<String> = vec![];

    for statement in statements {
        let pretty_statement = format_statement(sql_string.clone(), statement, check, max_width)?;
        pretty_statements.push(pretty_statement);
    }

    Ok(pretty_statements)
}
