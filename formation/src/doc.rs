use crate::error::{self, FormaError};
use pretty::RcDoc;
use sqlparser::ast::{
    BinaryOperator, Cte, Expr, Join, JoinConstraint, JoinOperator, Query, Select, SelectItem,
    SetExpr, Statement, TableAlias, TableFactor, TableWithJoins,
};

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
fn resolve_sub_expr<'a>(expr_string: String, negated: bool, body: RcDoc<'a, ()>) -> RcDoc<'a, ()> {
    resolve_negation(expr_string, negated).append(transform_sub_expr(body))
}

/// Transforms an `Expr` that appears as a sub-expression, e.g.
/// ` ...exists (...)`, to an `RcDoc`.
fn transform_sub_expr<'a>(body: RcDoc<'a, ()>) -> RcDoc<'a, ()> {
    RcDoc::nil()
        .append(
            RcDoc::text("(")
                .append(RcDoc::line_())
                .append(body)
                .nest(2)
                .append(RcDoc::line_())
                .append(RcDoc::text(")"))
                .group(),
        )
        .nest(2)
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
        _ => unreachable!("Unhandled `Expr`"),
    }
}

/// Transforms the given `Expr` into an `RcDoc`. If `expr` is `None`, returns
/// a nil `RcDoc`.
fn transform_expr<'a>(expr: Option<Expr>) -> RcDoc<'a, ()> {
    match expr {
        Some(expr) => match expr {
            Expr::BinaryOp { left, op, right } => transform_expr(Some(*left))
                // TODO: Rework to avoid trailing spaces.
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
            Expr::Exists(box query) => RcDoc::text("exists")
                .append(RcDoc::softline().append(transform_sub_expr(transform_query(query)))),
            Expr::Subquery(box query) => {
                RcDoc::softline_().append(transform_sub_expr(transform_query(query)))
            }
            // TODO: Handle other expression types.
            _ => RcDoc::text(expr.to_string()),
        },
        None => RcDoc::nil(),
    }
}

fn transform_join<'a>(join: Join) -> RcDoc<'a, ()> {
    fn prefix<'a>(constraint: &JoinConstraint) -> RcDoc<'a, ()> {
        RcDoc::text(match constraint {
            JoinConstraint::Natural => "natural",
            _ => "",
        })
    }

    fn suffix<'a>(constraint: &JoinConstraint) -> RcDoc<'a, ()> {
        match constraint {
            JoinConstraint::On(expr) => RcDoc::space().append(
                RcDoc::text("on").append(RcDoc::space().append(transform_expr(Some(expr.clone())))),
            ),
            // TODO:
            // JoinConstraint::Using(attrs) => &format!(" using ({})", display_comma_separated(attrs)),
            _ => RcDoc::nil(),
        }
    }

    match join.join_operator {
        JoinOperator::Inner(constraint) => RcDoc::text("join").append(
            RcDoc::space().append(
                prefix(&constraint)
                    .append(transform_relation(join.relation).append(suffix(&constraint))),
            ),
        ),
        JoinOperator::LeftOuter(constraint) => RcDoc::text("left join").append(
            RcDoc::space().append(
                prefix(&constraint)
                    .append(transform_relation(join.relation).append(suffix(&constraint))),
            ),
        ),
        JoinOperator::RightOuter(constraint) => RcDoc::text("right join").append(
            RcDoc::space().append(
                prefix(&constraint)
                    .append(transform_relation(join.relation).append(suffix(&constraint))),
            ),
        ),
        JoinOperator::FullOuter(constraint) => RcDoc::text("full join").append(
            RcDoc::space().append(
                prefix(&constraint)
                    .append(transform_relation(join.relation).append(suffix(&constraint))),
            ),
        ),
        JoinOperator::CrossJoin => RcDoc::text("cross join")
            .append(RcDoc::space().append(transform_relation(join.relation))),
        JoinOperator::CrossApply => RcDoc::text("cross apply")
            .append(RcDoc::space().append(transform_relation(join.relation))),
        JoinOperator::OuterApply => RcDoc::text("outer apply")
            .append(RcDoc::space().append(transform_relation(join.relation))),
    }
}

fn transform_args<'a>(args: Vec<Expr>) -> RcDoc<'a, ()> {
    if !args.is_empty() {
        // TODO: This is a pattern that's largely shared a in few places.
        RcDoc::text("(")
            .append(RcDoc::line_())
            .append(RcDoc::intersperse(
                args.iter().map(|expr| transform_expr(Some(expr.clone()))),
                RcDoc::text(",").append(RcDoc::space()),
            ))
            .nest(2)
            .append(RcDoc::line_())
            .append(RcDoc::text(")"))
            .group()
    } else {
        RcDoc::nil()
    }
}

fn transform_alias<'a>(alias: Option<TableAlias>) -> RcDoc<'a, ()> {
    if let Some(alias) = alias {
        RcDoc::space()
            .append(RcDoc::text(alias.to_string()))
            .append(RcDoc::space().append(RcDoc::text("as")))
    } else {
        RcDoc::nil()
    }
}

fn transform_relation<'a>(relation: TableFactor) -> RcDoc<'a, ()> {
    match relation {
        TableFactor::Table {
            name, alias, args, ..
        } => RcDoc::text(name.to_string())
            .append(transform_args(args))
            .append(transform_alias(alias)),
        // TODO: handle other `TableFactor` variants.
        _ => RcDoc::text(relation.to_string()),
    }
}

/// Transforms the given `SetExpr` into an `RcDoc`.
fn transform_set_expr<'a>(set_expr: SetExpr) -> RcDoc<'a, ()> {
    match set_expr {
        SetExpr::Select(box Select {
            projection,
            from,
            selection,
            distinct,
            having,
            group_by,
        }) => {
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
                                from.into_iter().map(|table_with_joins| {
                                    let TableWithJoins { joins, relation } = table_with_joins;
                                    transform_relation(relation).append(if !joins.is_empty() {
                                        RcDoc::hardline().append(RcDoc::intersperse(
                                            joins.iter().map(|join| transform_join(join.clone())),
                                            RcDoc::line(),
                                        ))
                                    } else {
                                        RcDoc::nil()
                                    })
                                }),
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
        }
        SetExpr::SetOperation {
            op,
            all,
            left,
            right,
        } => {
            transform_set_expr(*left)
                .append(RcDoc::hardline().append(
                    RcDoc::text(op.to_string().to_lowercase()).append(if all {
                        RcDoc::space().append(RcDoc::text("all"))
                    } else {
                        RcDoc::nil()
                    }),
                ))
                .append(RcDoc::hardline())
                .append(transform_set_expr(*right))
        }
        // Parenthensized query, i.e. order evaluation enforcement.
        // TODO: Is this generalizable?
        SetExpr::Query(query) => RcDoc::text("(")
            .append(RcDoc::line())
            .append(transform_query(*query))
            .nest(2)
            .append(RcDoc::line().append(RcDoc::text(")")))
            .group(),
        _ => unreachable!("Unhandled `SetExpr`"),
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
    // CTEs.
    let mut doc: RcDoc<'a, ()> = if !ctes.is_empty() {
        RcDoc::text("with")
            .append(RcDoc::intersperse(
                ctes.into_iter().map(|Cte { alias, query }| {
                    transform_alias(Some(alias))
                        .append(RcDoc::softline())
                        // Parenthensized query.
                        .append(
                            RcDoc::text("(")
                                .append(RcDoc::line_())
                                .append(transform_query(query))
                                .nest(2)
                                .append(RcDoc::line_())
                                .append(RcDoc::text(")"))
                                .group(),
                        )
                }),
                RcDoc::text(",").append(RcDoc::line()),
            ))
            .append(RcDoc::line().append(RcDoc::line()))
    } else {
        RcDoc::nil()
    };

    // Query body, e.g. `select * from t1 where x > 1`.
    doc = match body.to_owned() {
        SetExpr::Select(..) => doc.append(transform_set_expr(body)),
        SetExpr::SetOperation { .. } => doc.append(transform_set_expr(body)),
        _ => doc,
    };

    // Order by.
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

    // Limit.
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
        _ => unreachable!("Unhandled `Statement` variant"),
    }
}

/// Turns normal SQL into delightfully formatted SQL.
pub fn render_statement(statement: Statement, max_width: usize) -> error::Result<String> {
    let mut bs = Vec::new();
    transform_statement(statement)
        .render(max_width, &mut bs)
        .map_err(|op| FormaError::TransformationFailure(op))?;
    String::from_utf8(bs).map_err(|_| FormaError::Utf8Failure)
}
