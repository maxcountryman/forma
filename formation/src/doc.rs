use crate::error::{self, FormaError};
use pretty::RcDoc;
use sqlparser::ast::{
    BinaryOperator, Cte, Expr, Function, Join, JoinConstraint, JoinOperator, OrderByExpr, Query,
    Select, SelectItem, SetExpr, Statement, TableAlias, TableFactor, TableWithJoins, WindowFrame,
    WindowSpec,
};

/// Returns `true` if the given `BinaryOperator` should create a newline,
/// otherwise `false`.
fn is_newline_op(binop: &BinaryOperator) -> bool {
    *binop == BinaryOperator::And || *binop == BinaryOperator::Or
}

fn comma_separated<'a, D>(docs: D) -> RcDoc<'a, ()>
where
    D: Iterator<Item = RcDoc<'a, ()>>,
{
    RcDoc::intersperse(docs, RcDoc::text(",").append(RcDoc::line()))
}

fn parenthenized(doc: RcDoc<'_, ()>) -> RcDoc<'_, ()> {
    RcDoc::text("(")
        .append(RcDoc::line_())
        .append(doc)
        .nest(2)
        .append(RcDoc::line_())
        .append(RcDoc::text(")"))
        .group()
}

/// Resolves a possibly negated expression to an `RcDoc`.
fn resolve_negation<'a>(expr: Expr, negated: bool) -> RcDoc<'a, ()> {
    transform_expr(Some(expr))
        .append(RcDoc::space())
        .append(if negated {
            RcDoc::text("not").append(RcDoc::space())
        } else {
            RcDoc::nil()
        })
        .append(RcDoc::text("in"))
        .append(RcDoc::softline())
}

fn transform_select_item<'a>(select_item: SelectItem) -> RcDoc<'a, ()> {
    match select_item {
        SelectItem::ExprWithAlias { expr, alias } => transform_expr(Some(expr))
            .append(RcDoc::space())
            .append(RcDoc::text("as"))
            .append(RcDoc::space())
            .append(RcDoc::text(alias)),
        SelectItem::UnnamedExpr(expr) => transform_expr(Some(expr)),
        SelectItem::QualifiedWildcard(object_name) => RcDoc::text(object_name.to_string()),
        SelectItem::Wildcard => RcDoc::text("*"),
    }
}

/// Transforms the given `Expr` into an `RcDoc`. If `expr` is `None`, returns
/// a nil `RcDoc`.
fn transform_expr<'a>(expr: Option<Expr>) -> RcDoc<'a, ()> {
    match expr {
        Some(expr) => match expr {
            Expr::Identifier(ident) => RcDoc::text(ident),
            Expr::Wildcard => RcDoc::text("*"),
            Expr::QualifiedWildcard(qualifiers) => {
                RcDoc::intersperse(qualifiers, RcDoc::text(".")).append(RcDoc::text(".*"))
            }
            Expr::CompoundIdentifier(idents) => RcDoc::intersperse(idents, RcDoc::text(".")),
            Expr::BinaryOp { left, op, right } => {
                let op_string = op.to_string().to_lowercase();
                transform_expr(Some(*left))
                    .append(if is_newline_op(&op) {
                        RcDoc::hardline()
                            .append(RcDoc::text(op_string))
                            .append(RcDoc::space())
                    } else {
                        RcDoc::space().append(RcDoc::text(op_string).append(RcDoc::space()))
                    })
                    .append(transform_expr(Some(*right)))
            }
            Expr::UnaryOp { expr, op } => RcDoc::text(op.to_string().to_lowercase())
                .append(RcDoc::space())
                .append(transform_expr(Some(*expr))),
            Expr::Cast { expr, data_type } => RcDoc::text("cast")
                .append(RcDoc::text("("))
                .append(
                    transform_expr(Some(*expr))
                        .append(RcDoc::space())
                        .append(RcDoc::text("as"))
                        .append(RcDoc::text(data_type.to_string().to_lowercase())),
                )
                .append(RcDoc::text(")")),
            Expr::Extract { field, expr } => RcDoc::text("extract")
                .append(RcDoc::text("("))
                .append(
                    RcDoc::text(field.to_string())
                        .append(RcDoc::space())
                        .append(RcDoc::text("from"))
                        .append(RcDoc::space())
                        .append(transform_expr(Some(*expr))),
                )
                .append(RcDoc::text(")")),
            Expr::Collate { expr, collation } => transform_expr(Some(*expr))
                .append(RcDoc::space())
                .append(RcDoc::text("collate"))
                .append(RcDoc::space())
                .append(RcDoc::text(collation.to_string())),
            Expr::Nested(expr) => RcDoc::text("(")
                .append(RcDoc::softline_())
                .append(transform_expr(Some(*expr)).group())
                .nest(2)
                .append(RcDoc::softline_())
                .append(RcDoc::text(")")),
            Expr::Value(value) => RcDoc::text(value.to_string()),
            Expr::InSubquery {
                expr,
                negated,
                subquery,
            } => resolve_negation(*expr, negated).append(parenthenized(transform_query(*subquery))),
            Expr::InList {
                expr,
                negated,
                list,
            } => resolve_negation(*expr, negated).append(parenthenized(comma_separated(
                list.into_iter().map(|expr| transform_expr(Some(expr))),
            ))),
            Expr::Exists(box query) => RcDoc::text("exists")
                .append(RcDoc::softline().append(parenthenized(transform_query(query)))),
            Expr::Subquery(box query) => {
                RcDoc::softline_().append(parenthenized(transform_query(query)))
            }
            Expr::Between {
                expr,
                negated,
                low,
                high,
            } => transform_expr(Some(*expr))
                .append(if negated {
                    RcDoc::text("not")
                } else {
                    RcDoc::nil()
                })
                .append(
                    RcDoc::space().append(
                        RcDoc::text("between")
                            .append(RcDoc::space())
                            .append(transform_expr(Some(*low)))
                            .append(
                                RcDoc::space()
                                    .append(RcDoc::text("and"))
                                    .append(RcDoc::space()),
                            )
                            .append(transform_expr(Some(*high))),
                    ),
                ),
            Expr::Case {
                operand,
                conditions,
                results,
                else_result,
            } => RcDoc::text("case")
                .append(if let Some(operand) = operand {
                    RcDoc::space().append(transform_expr(Some(*operand)))
                } else {
                    RcDoc::nil()
                })
                .append(
                    RcDoc::line().nest(2).append(
                        RcDoc::intersperse(
                            conditions.iter().zip(results).map(|(condition, result)| {
                                RcDoc::text("when")
                                    .append(RcDoc::space())
                                    .append(transform_expr(Some(condition.clone())))
                                    .append(RcDoc::space())
                                    .append(RcDoc::text("then"))
                                    .append(RcDoc::space())
                                    .append(transform_expr(Some(result)))
                            }),
                            RcDoc::line(),
                        )
                        .append(if let Some(else_result) = else_result {
                            RcDoc::line().nest(2).append(
                                RcDoc::text("else")
                                    .append(RcDoc::space())
                                    .append(transform_expr(Some(*else_result))),
                            )
                        } else {
                            RcDoc::nil()
                        }),
                    ),
                )
                .append(RcDoc::line().append(RcDoc::text("end"))),
            Expr::IsNull(expr) => transform_expr(Some(*expr))
                .append(RcDoc::space())
                .append(RcDoc::text("is null")),
            Expr::IsNotNull(expr) => transform_expr(Some(*expr))
                .append(RcDoc::space())
                .append(RcDoc::text("is not null")),
            Expr::Function(Function {
                name,
                args,
                over,
                distinct,
            }) => RcDoc::text(name.to_string().to_lowercase())
                .append(parenthenized(
                    if distinct {
                        RcDoc::text("distinct").append(RcDoc::space())
                    } else {
                        RcDoc::nil()
                    }
                    .append(comma_separated(
                        args.into_iter().map(|expr| transform_expr(Some(expr))),
                    )),
                ))
                .append(
                    if let Some(WindowSpec {
                        partition_by,
                        order_by,
                        window_frame,
                    }) = over
                    {
                        RcDoc::space().append(
                            RcDoc::text("over").append(parenthenized(
                                if !partition_by.is_empty() {
                                    RcDoc::text("partition by")
                                        .append(RcDoc::space())
                                        .append(comma_separated(
                                            partition_by
                                                .into_iter()
                                                .map(|expr| transform_expr(Some(expr))),
                                        ))
                                        .append(RcDoc::space())
                                } else {
                                    RcDoc::nil()
                                }
                                .append(if !order_by.is_empty() {
                                    RcDoc::line_().append(
                                        RcDoc::text("order by").append(RcDoc::space()).append(
                                            comma_separated(order_by.into_iter().map(
                                                |OrderByExpr { expr, asc }| {
                                                    transform_expr(Some(expr)).append(
                                                        if let Some(asc) = asc {
                                                            RcDoc::space().append(if asc {
                                                                RcDoc::text("asc")
                                                            } else {
                                                                RcDoc::text("desc")
                                                            })
                                                        } else {
                                                            RcDoc::nil()
                                                        },
                                                    )
                                                },
                                            )),
                                        ),
                                    )
                                } else {
                                    RcDoc::nil()
                                })
                                .append(
                                    if let Some(WindowFrame {
                                        units,
                                        start_bound,
                                        end_bound,
                                    }) = window_frame
                                    {
                                        RcDoc::line_().append(
                                            RcDoc::text(units.to_string().to_lowercase())
                                                .append(RcDoc::space())
                                                .append(RcDoc::text("between"))
                                                .append(RcDoc::space())
                                                .append(start_bound.to_string().to_lowercase())
                                                .append(if let Some(end_bound) = end_bound {
                                                    RcDoc::space()
                                                        .append(RcDoc::text("and"))
                                                        .append(RcDoc::space())
                                                        .append(RcDoc::text(
                                                            end_bound.to_string().to_lowercase(),
                                                        ))
                                                } else {
                                                    RcDoc::nil()
                                                }),
                                        )
                                    } else {
                                        RcDoc::nil()
                                    },
                                ),
                            )),
                        )
                    } else {
                        RcDoc::nil()
                    },
                ),
        },
        None => RcDoc::nil(),
    }
}

fn transform_join<'a>(join: Join) -> RcDoc<'a, ()> {
    fn prefix<'a>(constraint: &JoinConstraint) -> RcDoc<'a, ()> {
        match constraint {
            JoinConstraint::Natural => RcDoc::text("natural").append(RcDoc::space()),
            _ => RcDoc::nil(),
        }
    }

    fn suffix<'a>(constraint: &JoinConstraint) -> RcDoc<'a, ()> {
        match constraint.clone() {
            JoinConstraint::On(expr) => RcDoc::space().append(
                RcDoc::text("on").append(RcDoc::space().append(transform_expr(Some(expr)))),
            ),
            JoinConstraint::Using(attrs) => {
                RcDoc::space().append(RcDoc::text("using").append(RcDoc::space()).append(
                    parenthenized(comma_separated(attrs.into_iter().map(RcDoc::text))),
                ))
            }
            _ => RcDoc::nil(),
        }
    }

    match join.join_operator {
        JoinOperator::Inner(constraint) => prefix(&constraint).append(RcDoc::text("join")).append(
            RcDoc::space().append(transform_relation(join.relation).append(suffix(&constraint))),
        ),
        JoinOperator::LeftOuter(constraint) => prefix(&constraint).append(
            RcDoc::text("left join").append(
                RcDoc::space()
                    .append(transform_relation(join.relation).append(suffix(&constraint))),
            ),
        ),
        JoinOperator::RightOuter(constraint) => prefix(&constraint).append(
            RcDoc::text("right join").append(
                RcDoc::space()
                    .append(transform_relation(join.relation).append(suffix(&constraint))),
            ),
        ),
        JoinOperator::FullOuter(constraint) => prefix(&constraint).append(
            RcDoc::text("full join").append(
                RcDoc::space()
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

fn transform_exprs<'a>(exprs: Vec<Expr>) -> RcDoc<'a, ()> {
    if !exprs.is_empty() {
        parenthenized(comma_separated(
            exprs.iter().map(|expr| transform_expr(Some(expr.clone()))),
        ))
    } else {
        RcDoc::nil()
    }
}

fn transform_alias<'a>(alias: Option<TableAlias>) -> RcDoc<'a, ()> {
    if let Some(alias) = alias {
        RcDoc::space()
            .append(RcDoc::text("as").append(RcDoc::space()))
            .append(RcDoc::text(alias.to_string()))
    } else {
        RcDoc::nil()
    }
}

fn transform_relation<'a>(relation: TableFactor) -> RcDoc<'a, ()> {
    match relation {
        TableFactor::Table {
            name,
            alias,
            args,
            // TODO: `with_hints` support.
            with_hints: _,
        } => RcDoc::text(name.to_string())
            .append(transform_exprs(args))
            .append(transform_alias(alias)),
        TableFactor::Derived {
            lateral,
            subquery,
            alias,
        } => RcDoc::text(if lateral { "lateral " } else { "" })
            .append(parenthenized(transform_query(*subquery)).append(transform_alias(alias))),
        // TODO: handle other `TableFactor` variants.
        _ => RcDoc::text(relation.to_string()),
    }
}

fn transform_order_by<'a>(order_by_expr: OrderByExpr) -> RcDoc<'a, ()> {
    let OrderByExpr { expr, asc } = order_by_expr;
    transform_expr(Some(expr)).append(if let Some(asc) = asc {
        RcDoc::line().append(if asc {
            RcDoc::text("asc")
        } else {
            RcDoc::text("desc")
        })
    } else {
        RcDoc::nil()
    })
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
                .append(comma_separated(
                    projection.into_iter().map(transform_select_item),
                ))
                .nest(2);

            // From.
            doc = if !from.is_empty() {
                doc.append(
                    RcDoc::hardline().append(RcDoc::text("from")).append(
                        RcDoc::line().nest(2).append(
                            comma_separated(from.into_iter().map(|table_with_joins| {
                                let TableWithJoins { joins, relation } = table_with_joins;
                                transform_relation(relation).append(if !joins.is_empty() {
                                    RcDoc::hardline().append(RcDoc::intersperse(
                                        joins.into_iter().map(transform_join),
                                        RcDoc::line(),
                                    ))
                                } else {
                                    RcDoc::nil()
                                })
                            }))
                            .nest(2)
                            .group(),
                        ),
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
                        .append(transform_expr(selection).nest(2)),
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
                            comma_separated(
                                group_by.into_iter().map(|expr| transform_expr(Some(expr))),
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
        SetExpr::Query(query) => parenthenized(transform_query(*query)),

        // Values, such as insert values for the given expression.
        SetExpr::Values(values) => {
            RcDoc::text("values")
                .append(RcDoc::space())
                .append(RcDoc::concat(values.0.into_iter().map(|row| {
                    parenthenized(comma_separated(
                        row.into_iter().map(|expr| transform_expr(Some(expr))),
                    ))
                })))
        }
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
            .append(comma_separated(ctes.into_iter().map(
                |Cte { alias, query }| {
                    RcDoc::space()
                        // Special-case CTEs alias handling.
                        .append(RcDoc::text(format!("{} as", alias.to_string())))
                        .append(RcDoc::softline())
                        .append(parenthenized(transform_query(query)))
                },
            )))
            .append(RcDoc::line().append(RcDoc::line()))
    } else {
        RcDoc::nil()
    }
    // Query body, e.g. `select * from t1 where x > 1`.
    .append(transform_set_expr(body));

    // Order by.
    doc = if !order_by.is_empty() {
        doc.append(
            RcDoc::line()
                .append(RcDoc::text("order by").append(RcDoc::line().nest(2)))
                .append(
                    RcDoc::intersperse(
                        order_by.into_iter().map(transform_order_by),
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

/// Renders the `Statement` in accordance with the provided maximum width.
pub fn render_statement(statement: Statement, max_width: usize) -> error::Result<String> {
    let mut bs = Vec::new();
    transform_statement(statement)
        .render(max_width, &mut bs)
        .map_err(FormaError::TransformationFailure)?;
    String::from_utf8(bs).map_err(|_| FormaError::Utf8Failure)
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
