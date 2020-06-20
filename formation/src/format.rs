//! The primary formatting interface
//!
//! This module provides a formatting function [`format`] which is intended to be used to format SQL
//! strings in an opinionated fashion. The function is only configurable in a minimal way by
//! design.
//!
//! [`format`]: ../format/fn.format.html

use sqlparser::ast::Statement;
use sqlparser::parser::Parser;

use crate::dialect::TemplatedDialect;
use crate::doc::render_statement;
use crate::error::{self, FormaError};

fn format_statement(
    sql: &str,
    statement: Statement,
    check: bool,
    max_width: usize,
) -> error::Result<String> {
    let pretty = format!("{};\n", render_statement(statement, max_width)?);
    if check && pretty != sql {
        Err(FormaError::WouldFormat)
    } else {
        Ok(pretty)
    }
}

/// Formats a given SQL string in accordance with the given maximum width.
///
/// Each statement parsed is formatted separately. The result is a `Vec<String>` where each item
/// represents a formatted statement of the original `sql_string` input.
///
///
///
/// # Errors
///
/// Returns a [`FormaError::InvalidInput`] if the parser cannot parse the provided input.
///
/// If `check` is `true`, will return a [`FormaError::WouldFormat`] if the provided input would be
/// formatted.
///
/// [`FormaError::InvalidInput`]: ../error/enum.FormaError.html#variant.InvalidInput
/// [`FormaError::WouldFormat`]: ../error/enum.FormaError.html#variant.WouldFormat
///
/// # Example
///
/// ```
/// use formation::format;
/// let sql = "SELECT * FROM users;";
/// assert_eq!(
///     format(sql, false, 100).unwrap(),
///     vec!["select * from users;\n".to_owned()]
/// );
/// ```
pub fn format(sql: &str, check: bool, max_width: usize) -> error::Result<Vec<String>> {
    let dialect = TemplatedDialect {};
    let statements = Parser::parse_sql(&dialect, sql)?;
    let mut pretty_statements: Vec<String> = vec![];

    for statement in statements {
        let pretty_statement = format_statement(sql, statement, check, max_width)?;
        pretty_statements.push(pretty_statement);
    }

    Ok(pretty_statements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use sqlparser::ast::{Expr, Query, Select, SelectItem, SetExpr, Value};

    const MAX_WIDTH: usize = 100;

    #[test]
    fn test_format_statement() {
        let sql_string = "SELECT 42;";
        let statement = Statement::Query(Box::new(Query {
            body: SetExpr::Select(Box::new(Select {
                distinct: false,
                from: vec![],
                group_by: vec![],
                having: None,
                projection: vec![SelectItem::UnnamedExpr(Expr::Value(Value::Number(
                    42.to_string(),
                )))],
                selection: None,
                top: None,
            })),
            ctes: vec![],
            fetch: None,
            limit: None,
            offset: None,
            order_by: vec![],
        }));
        assert_eq!(
            format_statement(sql_string, statement, false, MAX_WIDTH).unwrap(),
            "select 42;\n".to_owned()
        );
    }

    #[test]
    fn test_format() {
        let sql_string = "select id from users where created_at > {{date}};".to_owned();
        assert_eq!(
            format(&sql_string, false, MAX_WIDTH).unwrap(),
            vec!["select id from users where created_at > {{date}};\n".to_owned()]
        );
    }

    #[test]
    fn test_format_check() {
        let sql_string = "select * from t1";
        let result = format(sql_string, true, MAX_WIDTH);
        dbg!(&result);
        assert_eq!(result.is_err(), true);
    }
}
