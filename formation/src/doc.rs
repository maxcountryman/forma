use crate::FormaError;
use pretty::RcDoc;
use sqlparser::ast::{BinaryOperator, Cte, Expr, SelectItem};
use sqlparser::ast::{Query, Select, SetExpr, Statement};

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

/// Transforms the given `SetExpr` if it's `SetExpr::Select`.
fn transform_select<'a>(set_expr: SetExpr) -> RcDoc<'a, ()> {
    if let SetExpr::Select(box Select {
        projection,
        from,
        selection,
        distinct,
        having,
        group_by,
    }) = set_expr
    {
        let mut doc = RcDoc::text("select")
            .append(if distinct {
                RcDoc::space().append(RcDoc::text("disinct"))
            } else {
                RcDoc::nil()
            })
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
        doc = if selection.is_some() {
            doc.append(
                RcDoc::line()
                    .append(RcDoc::text("where").append(RcDoc::line().nest(2)))
                    .append(transform_expr(selection)),
            )
        } else {
            doc
        };

        // Group By.
        doc = if !group_by.is_empty() {
            doc.append(
                RcDoc::line()
                    .append(RcDoc::text("group by").append(RcDoc::line().nest(2)))
                    .append(
                        RcDoc::intersperse(
                            group_by.into_iter().map(|x| x.to_string()),
                            RcDoc::text(",").append(RcDoc::line()),
                        )
                        .nest(2)
                        .group(),
                    ),
            )
        } else {
            doc
        };

        // Having.
        if having.is_some() {
            doc.append(
                RcDoc::line()
                    .append(RcDoc::text("having").append(RcDoc::line().nest(2)))
                    .append(transform_expr(having)),
            )
        } else {
            doc
        }
    } else {
        panic!("Improper `SetExpr` variant provided; must be `SetExpr::Select`")
    }
}

/// Transforms the given `Query` into an `RcDoc`.
fn transform_query<'a>(query: Query) -> RcDoc<'a, ()> {
    let Query {
        body,
        order_by,
        limit,
        ctes,
        offset: _,
        fetch: _,
    } = query;
    let mut doc: RcDoc<'a, ()> = if !ctes.is_empty() {
        RcDoc::text("with")
            .append(RcDoc::line().append(RcDoc::intersperse(
                ctes.into_iter().map(|Cte { alias, query }| {
                    dbg!(&query);
                    RcDoc::text(alias.to_string())
                        .append(RcDoc::space())
                        .append(RcDoc::text("as"))
                        .append(RcDoc::line())
                        .append(transform_query(query))
                }),
                RcDoc::line(),
            )))
            .append(RcDoc::line())
    } else {
        RcDoc::nil()
    };

    doc = match body.to_owned() {
        SetExpr::Select(..) => doc.append(transform_select(body)),
        SetExpr::SetOperation {
            op,
            all: _,
            left,
            right,
        } => transform_select(*left)
            .append(RcDoc::text(op.to_string().to_lowercase()))
            .append(transform_select(*right)),
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
pub fn render_statement(statement: Statement, max_width: usize) -> Result<String, FormaError> {
    let mut bs = Vec::new();
    transform_statement(statement)
        .render(max_width, &mut bs)
        .map_err(|op| FormaError::TransformationFailure(op))?;
    String::from_utf8(bs).map_err(|_| FormaError::Utf8Failure)
}
