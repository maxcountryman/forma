mod body;
mod common;
mod expr;
mod query;

use crate::error;
use sqlparser::ast::Statement;

use crate::doc::common::FormaDoc;
use crate::doc::query::query_doc;

/// Transforms the given `Statement` into an `RcDoc`.
fn statement_doc<'a>(statement: Statement) -> FormaDoc<'a> {
    match statement {
        // Select statement.
        Statement::Query(query) => query_doc(*query),
        // TODO: Match remaining statement variants.
        _ => unreachable!("Unhandled `Statement` variant"),
    }
}

/// Renders the `Statement` in accordance with the provided maximum width.
pub fn render_statement(statement: Statement, max_width: usize) -> error::Result<String> {
    let mut bs = Vec::new();
    statement_doc(statement).render(max_width, &mut bs)?;
    Ok(String::from_utf8(bs)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use sqlparser::ast::{Expr, Query, Select, SelectItem, SetExpr, Value};

    const MAX_WIDTH: usize = 100;

    #[test]
    fn test_render_statement() {
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
            render_statement(statement, MAX_WIDTH).unwrap(),
            "select 42".to_owned()
        );
    }
}
