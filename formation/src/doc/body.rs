use pretty::RcDoc;
use sqlparser::ast::{
    Expr, Join, JoinConstraint, JoinOperator, Select, SelectItem, SetExpr, TableAlias, TableFactor,
    TableWithJoins, Top,
};

use crate::constants::NEST_FACTOR;
use crate::doc::common::{ident_doc, interweave_comma, parenthenized, Exprs, FormaDoc};
use crate::doc::expr::expr_doc;
use crate::doc::query::query_doc;

/// Transforms the given `SetExpr` into an `RcDoc`.
pub fn body_doc<'a>(body: SetExpr) -> FormaDoc<'a> {
    match body {
        SetExpr::Select(box Select {
            distinct,
            top,
            projection,
            from,
            selection,
            group_by,
            having,
        }) => {
            // Distinct.
            if distinct {
                RcDoc::text("select distinct")
            } else {
                RcDoc::text("select")
            }
            // Top.
            .append(top_doc(top))
            // Projection.
            .append(projection_doc(projection))
            // From.
            .append(from_doc(from))
            // Selection.
            .append(selection_doc(selection))
            // Group By.
            .append(group_by_doc(group_by))
            // Having.
            .append(having_doc(having))
        }

        SetExpr::SetOperation {
            op,
            all,
            left,
            right,
        } => body_doc(*left)
            .append(
                RcDoc::line().append(RcDoc::text(op.to_string().to_lowercase()).append(if all {
                    RcDoc::space().append(RcDoc::text("all"))
                } else {
                    RcDoc::nil()
                })),
            )
            .append(RcDoc::line())
            .append(body_doc(*right)),

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

fn projection_doc<'a>(projection: Vec<SelectItem>) -> FormaDoc<'a> {
    RcDoc::line().nest(NEST_FACTOR).append(
        interweave_comma(projection.into_iter().map(|select_item| {
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
        }))
        .nest(NEST_FACTOR)
        .group(),
    )
}

fn from_doc<'a>(from: Vec<TableWithJoins>) -> FormaDoc<'a> {
    if !from.is_empty() {
        RcDoc::line().append(RcDoc::text("from")).append(
            RcDoc::line().nest(NEST_FACTOR).append(
                interweave_comma(from.into_iter().map(|TableWithJoins { joins, relation }| {
                    relation_doc(relation).append(if !joins.is_empty() {
                        RcDoc::line().append(RcDoc::intersperse(
                            joins.into_iter().map(join_doc),
                            RcDoc::line(),
                        ))
                    } else {
                        RcDoc::nil()
                    })
                }))
                .nest(NEST_FACTOR)
                .group(),
            ),
        )
    } else {
        RcDoc::nil()
    }
}

fn group_by_doc<'a>(group_by: Exprs) -> FormaDoc<'a> {
    if !group_by.is_empty() {
        RcDoc::line()
            .append(RcDoc::text("group by").append(RcDoc::line().nest(NEST_FACTOR)))
            .append(
                interweave_comma(group_by.into_iter().map(expr_doc))
                    .nest(NEST_FACTOR)
                    .group(),
            )
    } else {
        RcDoc::nil()
    }
}

fn selection_doc<'a>(selection: Option<Expr>) -> FormaDoc<'a> {
    if let Some(selection) = selection {
        RcDoc::line().append(RcDoc::text("where")).append(
            RcDoc::line()
                .nest(NEST_FACTOR)
                .append(expr_doc(selection).nest(NEST_FACTOR).group()),
        )
    } else {
        RcDoc::nil()
    }
}

fn having_doc<'a>(having: Option<Expr>) -> FormaDoc<'a> {
    if let Some(having) = having {
        RcDoc::line()
            .append(RcDoc::text("having").append(RcDoc::line().nest(NEST_FACTOR)))
            .append(expr_doc(having))
    } else {
        RcDoc::nil()
    }
}

fn exprs_doc<'a>(exprs: Exprs) -> FormaDoc<'a> {
    if !exprs.is_empty() {
        parenthenized(interweave_comma(exprs.into_iter().map(expr_doc)))
    } else {
        RcDoc::nil()
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
