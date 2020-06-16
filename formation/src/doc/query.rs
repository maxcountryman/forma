use pretty::RcDoc;
use sqlparser::ast::{
    Cte, Fetch, Join, JoinConstraint, JoinOperator, Offset, OffsetRows, Query, Select, SelectItem,
    SetExpr, TableAlias, TableFactor, TableWithJoins, Top,
};

use crate::constants::NEST_FACTOR;
use crate::doc::common::{interweave_comma, ident_doc, order_by_doc, parenthenized, FormaDoc};
use crate::doc::expr::{expr_doc, exprs_doc};

/// Transforms the given `Query` into an `RcDoc`.
pub fn query_doc<'a>(
    Query {
        body,
        order_by,
        limit,
        ctes,
        offset,
        fetch,
    }: Query,
) -> FormaDoc<'a> {
    // CTEs.
    if !ctes.is_empty() {
        RcDoc::text("with")
            .append(RcDoc::space())
            .append(interweave_comma(ctes.into_iter().map(
                |Cte { alias, query }| {
                    // Special-case CTEs alias handling.
                    RcDoc::text(format!("{} as", alias.to_string()))
                        .append(RcDoc::softline())
                        .append(parenthenized(query_doc(query)))
                },
            )))
            .nest(NEST_FACTOR)
            .append(RcDoc::line().append(RcDoc::line()))
    } else {
        RcDoc::nil()
    }
    // Query body, e.g. `select * from t1 where x > 1`.
    .append(set_expr_doc(body))
    // Order by.
    .append(if !order_by.is_empty() {
        RcDoc::line()
            .append(RcDoc::text("order by").append(RcDoc::line().nest(NEST_FACTOR)))
            .append(
                interweave_comma(order_by.into_iter().map(order_by_doc))
                    .nest(NEST_FACTOR)
                    .group(),
            )
    } else {
        RcDoc::nil()
    })
    // Limit.
    .append(if let Some(limit) = limit {
        RcDoc::line()
            .append(RcDoc::text("limit").append(RcDoc::line().nest(NEST_FACTOR)))
            .append(RcDoc::text(limit.to_string()))
    } else {
        RcDoc::nil()
    })
    // Offset.
    .append(if let Some(offset) = offset {
        RcDoc::line().append(offset_doc(offset))
    } else {
        RcDoc::nil()
    })
    // Fetch.
    .append(
        if let Some(Fetch {
            with_ties,
            percent,
            quantity,
        }) = fetch
        {
            let extension = if with_ties {
                RcDoc::text("with ties")
            } else {
                RcDoc::text("only")
            };
            RcDoc::line().append(if let Some(quantity) = quantity {
                let percent = if percent {
                    RcDoc::space().append(RcDoc::text("percent"))
                } else {
                    RcDoc::nil()
                };
                RcDoc::text("fetch first")
                    .append(RcDoc::space())
                    .append(expr_doc(quantity))
                    .append(percent)
                    .append(RcDoc::space())
                    .append(RcDoc::text("rows"))
                    .append(RcDoc::space())
                    .append(extension)
            } else {
                RcDoc::text("fetch first rows")
                    .append(RcDoc::space())
                    .append(extension)
            })
        } else {
            RcDoc::nil()
        },
    )
    .group()
}
/// Transforms the given `SetExpr` into an `RcDoc`.
fn set_expr_doc<'a>(set_expr: SetExpr) -> FormaDoc<'a> {
    match set_expr {
        SetExpr::Select(box Select {
            projection,
            from,
            selection,
            distinct,
            having,
            group_by,
            top,
        }) => {
            if distinct {
                RcDoc::text("select distinct")
            } else {
                RcDoc::text("select")
            }
            .append(top_doc(top))
            .append(
                RcDoc::line().nest(NEST_FACTOR).append(
                    interweave_comma(projection.into_iter().map(select_item_doc))
                        .nest(NEST_FACTOR)
                        .group(),
                ),
            )
            // From.
            .append(if !from.is_empty() {
                RcDoc::line().append(RcDoc::text("from")).append(
                    RcDoc::line().nest(NEST_FACTOR).append(
                        interweave_comma(from.into_iter().map(
                            |TableWithJoins { joins, relation }| {
                                relation_doc(relation).append(if !joins.is_empty() {
                                    RcDoc::line().append(RcDoc::intersperse(
                                        joins.into_iter().map(join_doc),
                                        RcDoc::line(),
                                    ))
                                } else {
                                    RcDoc::nil()
                                })
                            },
                        ))
                        .nest(NEST_FACTOR)
                        .group(),
                    ),
                )
            } else {
                RcDoc::nil()
            })
            // Selection.
            .append(if let Some(selection) = selection {
                RcDoc::line().append(RcDoc::text("where")).append(
                    RcDoc::line()
                        .nest(NEST_FACTOR)
                        .append(expr_doc(selection).nest(NEST_FACTOR).group()),
                )
            } else {
                RcDoc::nil()
            })
            // Group By.
            .append(if !group_by.is_empty() {
                RcDoc::line()
                    .append(RcDoc::text("group by").append(RcDoc::line().nest(NEST_FACTOR)))
                    .append(
                        interweave_comma(group_by.into_iter().map(expr_doc))
                            .nest(NEST_FACTOR)
                            .group(),
                    )
            } else {
                RcDoc::nil()
            })
            // Having.
            .append(if let Some(having) = having {
                RcDoc::line()
                    .append(RcDoc::text("having").append(RcDoc::line().nest(NEST_FACTOR)))
                    .append(expr_doc(having))
            } else {
                RcDoc::nil()
            })
        }

        SetExpr::SetOperation {
            op,
            all,
            left,
            right,
        } => set_expr_doc(*left)
            .append(
                RcDoc::line().append(RcDoc::text(op.to_string().to_lowercase()).append(if all {
                    RcDoc::space().append(RcDoc::text("all"))
                } else {
                    RcDoc::nil()
                })),
            )
            .append(RcDoc::line())
            .append(set_expr_doc(*right)),

        // Parenthensized query, i.e. order evaluation enforcement.
        SetExpr::Query(query) => parenthenized(query_doc(*query)),

        // Values, such as insert values for the given expression.
        SetExpr::Values(values) => {
            RcDoc::text("values")
                .append(RcDoc::space())
                .append(RcDoc::concat(values.0.into_iter().map(|row| {
                    parenthenized(interweave_comma(row.into_iter().map(expr_doc)))
                })))
        }
    }
}

fn select_item_doc<'a>(select_item: SelectItem) -> FormaDoc<'a> {
    match select_item {
        SelectItem::ExprWithAlias { expr, alias } => expr_doc(expr)
            .append(RcDoc::space())
            .append(RcDoc::text("as"))
            .append(RcDoc::space())
            .append(RcDoc::text(alias.to_string())),
        SelectItem::QualifiedWildcard(object_name) => RcDoc::text(object_name.to_string()),
        SelectItem::UnnamedExpr(expr) => expr_doc(expr),
        SelectItem::Wildcard => RcDoc::text("*"),
    }
}

fn join_doc<'a>(join: Join) -> FormaDoc<'a> {
    fn prefix<'a>(constraint: &JoinConstraint) -> FormaDoc<'a> {
        match constraint {
            JoinConstraint::Natural => RcDoc::text("natural").append(RcDoc::space()),
            _ => RcDoc::nil(),
        }
    }

    fn suffix<'a>(constraint: &JoinConstraint) -> FormaDoc<'a> {
        match constraint.clone() {
            JoinConstraint::On(expr) => RcDoc::line()
                .append(RcDoc::text("on").append(RcDoc::space().append(expr_doc(expr))))
                .group(),
            JoinConstraint::Using(attrs) => RcDoc::line()
                .append(
                    RcDoc::text("using")
                        .append(RcDoc::space())
                        .append(parenthenized(interweave_comma(
                            attrs.into_iter().map(ident_doc),
                        ))),
                )
                .group(),
            _ => RcDoc::nil(),
        }
    }

    match join.join_operator {
        JoinOperator::Inner(constraint) => prefix(&constraint).append(RcDoc::text("join").append(
            RcDoc::space().append(relation_doc(join.relation).append(suffix(&constraint))),
        )),
        JoinOperator::LeftOuter(constraint) => {
            prefix(&constraint).append(RcDoc::text("left join").append(
                RcDoc::space().append(relation_doc(join.relation).append(suffix(&constraint))),
            ))
        }
        JoinOperator::RightOuter(constraint) => {
            prefix(&constraint).append(RcDoc::text("right join").append(
                RcDoc::space().append(relation_doc(join.relation).append(suffix(&constraint))),
            ))
        }
        JoinOperator::FullOuter(constraint) => {
            prefix(&constraint).append(RcDoc::text("full join").append(
                RcDoc::space().append(relation_doc(join.relation).append(suffix(&constraint))),
            ))
        }
        JoinOperator::CrossJoin => {
            RcDoc::text("cross join").append(RcDoc::space().append(relation_doc(join.relation)))
        }
        JoinOperator::CrossApply => {
            RcDoc::text("cross apply").append(RcDoc::space().append(relation_doc(join.relation)))
        }
        JoinOperator::OuterApply => {
            RcDoc::text("outer apply").append(RcDoc::space().append(relation_doc(join.relation)))
        }
    }
}

fn alias_doc<'a>(alias: Option<TableAlias>) -> FormaDoc<'a> {
    if let Some(alias) = alias {
        RcDoc::space()
            .append(RcDoc::text("as").append(RcDoc::space()))
            .append(RcDoc::text(alias.to_string()))
    } else {
        RcDoc::nil()
    }
}

fn relation_doc<'a>(relation: TableFactor) -> FormaDoc<'a> {
    match relation {
        TableFactor::Table {
            name,
            alias,
            args,
            with_hints,
        } => RcDoc::text(name.to_string())
            .append(exprs_doc(args))
            .append(alias_doc(alias))
            .append(if !with_hints.is_empty() {
                RcDoc::space().append(RcDoc::text("with").append(RcDoc::space()).append(
                    parenthenized(interweave_comma(with_hints.into_iter().map(expr_doc))),
                ))
            } else {
                RcDoc::nil()
            }),
        TableFactor::Derived {
            lateral,
            subquery,
            alias,
        } => RcDoc::text(if lateral { "lateral " } else { "" })
            .append(parenthenized(query_doc(*subquery)).append(alias_doc(alias))),
        TableFactor::NestedJoin(box TableWithJoins { relation, joins }) => {
            relation_doc(relation).append(RcDoc::concat(joins.into_iter().map(join_doc)))
        }
    }
}

fn top_doc<'a>(top: Option<Top>) -> FormaDoc<'a> {
    if let Some(Top {
        with_ties,
        percent,
        quantity,
    }) = top
    {
        let extension = if with_ties { " with ties" } else { "" };
        if let Some(quantity) = quantity {
            let percent = if percent { " percent" } else { "" };
            RcDoc::text(format!(" top ({}{}{})", quantity, percent, extension))
        } else {
            RcDoc::text(format!(" top{}", extension))
        }
    } else {
        RcDoc::nil()
    }
}

fn offset_doc<'a>(Offset { value, rows }: Offset) -> FormaDoc<'a> {
    RcDoc::text(format!("offset {}", value)).append(match rows {
        OffsetRows::None => RcDoc::nil(),
        OffsetRows::Row => RcDoc::text(" row"),
        OffsetRows::Rows => RcDoc::text(" rows"),
    })
}
