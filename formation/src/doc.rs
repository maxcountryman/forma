use crate::TemplatedDialect;
use pretty::RcDoc;
use sqlparser::ast::{BinaryOperator, Expr, SelectItem};
use sqlparser::ast::{Query, Select, SetExpr, Statement};
use sqlparser::parser::Parser;
use std::io;

pub enum FormaError {
    InvalidInput,
}

/// Returns `true` if the given `BinaryOperator` should create a newline,
/// otherwise `false`.
fn is_newline_op(binop: &BinaryOperator) -> bool {
    *binop == BinaryOperator::And || *binop == BinaryOperator::Or
}

/// Processes a `SelectItem`.
fn render_select_item(select_item: SelectItem) -> String {
    if let SelectItem::ExprWithAlias { expr, alias } = select_item {
        format!("{} as {}", expr, alias)
    } else {
        select_item.to_string()
    }
}

/// Resolves a possibly negated expression to an `RcDoc`.
fn resolve_negation<'a>(expr: String, negated: bool) -> RcDoc<'a, ()> {
    RcDoc::text(expr)
        .append(RcDoc::space())
        .append(if negated {
            RcDoc::text("not").append(RcDoc::space())
        } else {
            RcDoc::nil()
        })
        .append(RcDoc::text("in"))
        .append(RcDoc::softline())
}

/// Resolves a sub-expression, such as `InSquery` or `InLint` to an `RcDoc`.
fn resolve_sub_expr<'a>(expr_string: String, negated: bool, doc: RcDoc<'a, ()>) -> RcDoc<'a, ()> {
    resolve_negation(expr_string, negated)
        .append(
            RcDoc::text("(")
                .append(RcDoc::line_())
                .append(doc)
                .nest(2)
                .append(RcDoc::line_())
                .append(RcDoc::text(")"))
                .group(),
        )
        .nest(4)
}

/// Processes an `Expr` that operates over a list-like structure.
fn process_in_expr<'a>(expr: Expr) -> RcDoc<'a, ()> {
    match expr {
        Expr::InSubquery {
            expr,
            negated,
            subquery,
        } => resolve_sub_expr(expr.to_string(), negated, transform_query(*subquery)),
        Expr::InList {
            expr,
            negated,
            list,
        } => resolve_sub_expr(
            expr.to_string(),
            negated,
            RcDoc::intersperse(
                list.into_iter().map(|x| x.to_string()),
                RcDoc::text(",").append(RcDoc::line()),
            ),
        ),
        _ => RcDoc::nil(),
    }
}

/// Transforms the given `Expr` into an `RcDoc`.
fn transform_expr<'a>(expr: Option<Expr>) -> RcDoc<'a, ()> {
    match expr {
        Some(expr) => match expr {
            Expr::BinaryOp { left, op, right } => transform_expr(Some(*left))
                .append(RcDoc::space())
                .append(if is_newline_op(&op) {
                    RcDoc::hardline()
                        .append(RcDoc::text(op.to_string().to_lowercase()))
                        .append(RcDoc::space())
                        .nest(2)
                } else {
                    RcDoc::text(op.to_string()).append(RcDoc::space())
                })
                .append(transform_expr(Some(*right))),
            Expr::InSubquery { .. } => process_in_expr(expr),
            Expr::InList { .. } => process_in_expr(expr),
            // TODO: Handle other expression types.
            _ => RcDoc::text(expr.to_string()),
        },
        None => RcDoc::nil(),
    }
}

/// Transforms the given `Query` into an `RcDoc`.
fn transform_query<'a>(query: Query) -> RcDoc<'a, ()> {
    let Query {
        body,
        order_by,
        limit,
        ..
    } = query;
    // TODO: Match body on type, e.g. Select.
    let mut doc: RcDoc<'a, ()> = RcDoc::text("select");

    doc = match body.to_owned() {
        SetExpr::Select(box Select {
            projection,
            from,
            selection,
            ..
        }) => {
            // Projection.
            doc = doc
                .append(RcDoc::line())
                .append(RcDoc::intersperse(
                    projection
                        .into_iter()
                        .map(|select_item| render_select_item(select_item)),
                    RcDoc::text(",").append(RcDoc::line()),
                ))
                .nest(2);

            // From.
            doc = if !from.is_empty() {
                doc.append(
                    RcDoc::hardline()
                        .append(RcDoc::text("from").append(RcDoc::line().nest(2)))
                        .append(
                            RcDoc::intersperse(
                                from.into_iter().map(|x| x.to_string()),
                                RcDoc::text(",").append(RcDoc::line()),
                            )
                            .nest(2)
                            .group(),
                        ),
                )
            } else {
                doc
            };

            // Selection.
            if let Some(selection) = selection {
                doc.append(
                    RcDoc::line()
                        .append(RcDoc::text("where").append(RcDoc::line().nest(2)))
                        .append(transform_expr(Some(selection))),
                )
            } else {
                doc
            }
        }
        _ => doc,
    };

    doc = if !order_by.is_empty() {
        doc.append(
            RcDoc::line()
                .append(RcDoc::text("order by").append(RcDoc::line().nest(2)))
                .append(
                    RcDoc::intersperse(
                        order_by.into_iter().map(|x| x.to_string()),
                        RcDoc::text(",").append(RcDoc::line()),
                    )
                    .nest(2)
                    .group(),
                ),
        )
    } else {
        doc
    };

    if let Some(limit) = limit {
        doc.append(
            RcDoc::line()
                .append(RcDoc::text("limit").append(RcDoc::line().nest(2)))
                .append(RcDoc::text(limit.to_string())),
        )
    } else {
        doc
    }
    .group()
}

/// Transforms the given `Statement` into an `RcDoc`.
fn transform_statement<'a>(statement: Statement) -> RcDoc<'a, ()> {
    match statement {
        // Select statement.
        Statement::Query(query) => transform_query(*query),
        // TODO: Match remaining statement variants.
        _ => unreachable!(),
    }
}

/// Turns normal SQL into delightfully formatted SQL.
pub fn render_statement(statement: Statement, max_width: usize) -> io::Result<String> {
    let mut bs = Vec::new();
    transform_statement(statement).render(max_width, &mut bs)?;
    String::from_utf8(bs).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Unable to decode transformed bytes as UTF8",
        )
    })
}

/// TODO(bradford): notes
pub fn prettify(input_string: String, max_width: usize) -> Result<Vec<String>, FormaError> {
    let dialect = TemplatedDialect {};
    let statements =
        Parser::parse_sql(&dialect, input_string.clone()).map_err(|_| FormaError::InvalidInput)?;

    // This works but could panic, is Result<Vec<String>, Vec<FormaError>> annoying?
    // because we could take the errors out and stack them into a Vec
    let prettified = statements.into_iter().map(|statement| prettify_statement(statement, max_width).unwrap()).collect();
    Ok(prettified)
}
