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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::ast::{Expr, Query, Select, SelectItem, SetExpr, Value};

    const MAX_WIDTH: usize = 100;

    #[test]
    fn test_format_statement() {
        let sql_string = "SELECT 42;".to_owned();
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
            })),
            ctes: vec![],
            fetch: None,
            limit: None,
            offset: None,
            order_by: vec![],
        }));
        assert_eq!(
            format_statement(sql_string, statement, false, MAX_WIDTH).unwrap(),
            "select 42".to_owned()
        );
    }

    #[test]
    fn test_format() {
        let sql_string = "select id from users where created_at > {{date}};".to_owned();
        assert_eq!(
            format(sql_string, false, MAX_WIDTH).unwrap(),
            vec!["select\n  id\nfrom\n  users\nwhere\n  created_at > {{date}}".to_owned()]
        );
    }
}
