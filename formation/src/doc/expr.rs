use pretty::RcDoc;
use sqlparser::ast::{
    BinaryOperator, DataType, DateTimeField, Expr, Function, ListAgg, ListAggOnOverflow,
    ObjectName, Query, UnaryOperator, Value, WindowFrame, WindowSpec,
};

use crate::constants::NEST_FACTOR;
use crate::doc::common::{
    ident_doc, interweave_comma, order_by_doc, parenthenized, Exprs, FormaDoc, Idents,
};
use crate::doc::query::query_doc;

/// Returns a doc from the given `Expr`.
pub fn expr_doc<'a>(expr: Expr) -> FormaDoc<'a> {
    match expr {
        Expr::Between {
            expr,
            negated,
            low,
            high,
        } => between_doc(*expr, negated, *low, *high),
        Expr::BinaryOp { left, op, right } => binary_op_doc(op, *left, *right),
        Expr::Case {
            operand,
            conditions,
            results,
            else_result,
        } => case_doc(operand, conditions, results, else_result),
        Expr::Cast { expr, data_type } => cast_doc(*expr, data_type),
        Expr::Collate { expr, collation } => collate_doc(*expr, collation),
        Expr::CompoundIdentifier(idents) => compound_identifier_doc(idents),
        Expr::Exists(box query) => exists_doc(query),
        Expr::Extract { field, expr } => extract_doc(field, *expr),
        Expr::Function(Function {
            name,
            args,
            over,
            distinct,
        }) => function_doc(name, args, over, distinct),
        Expr::Identifier(ident) => ident_doc(ident),
        Expr::InList {
            expr,
            negated,
            list,
        } => in_list_doc(*expr, negated, list),
        Expr::InSubquery {
            expr,
            negated,
            subquery,
        } => in_subquery_doc(*expr, negated, *subquery),
        Expr::IsNotNull(expr) => is_not_null_doc(*expr),
        Expr::IsNull(expr) => is_null_doc(*expr),
        Expr::ListAgg(listagg) => listagg_doc(listagg),
        Expr::Nested(expr) => nested_doc(*expr),
        Expr::QualifiedWildcard(idents) => qualified_wildcard_doc(idents),
        Expr::Subquery(box query) => subquery_doc(query),
        Expr::TypedString { data_type, value } => typed_string_doc(data_type, value),
        Expr::UnaryOp { expr, op } => unary_op_doc(op, *expr),
        Expr::Value(value) => value_doc(value),
        Expr::Wildcard => RcDoc::text("*"),
    }
}

fn between_doc<'a>(expr: Expr, negated: bool, low: Expr, high: Expr) -> FormaDoc<'a> {
    expr_doc(expr)
        .append(if negated {
            RcDoc::text("not")
        } else {
            RcDoc::nil()
        })
        .append(
            RcDoc::space().append(
                RcDoc::text("between")
                    .append(RcDoc::space())
                    .append(expr_doc(low))
                    .append(
                        RcDoc::space()
                            .append(RcDoc::text("and"))
                            .append(RcDoc::space()),
                    )
                    .append(expr_doc(high)),
            ),
        )
}

fn binary_op_doc<'a>(op: BinaryOperator, left: Expr, right: Expr) -> FormaDoc<'a> {
    let op_string = op.to_string().to_lowercase();
    expr_doc(left)
        .append(if is_newline_op(&op) {
            RcDoc::line()
                .append(RcDoc::text(op_string))
                .append(RcDoc::space())
        } else {
            RcDoc::space().append(RcDoc::text(op_string).append(RcDoc::space()))
        })
        .append(expr_doc(right))
}

fn case_doc<'a>(
    operand: Option<Box<Expr>>,
    conditions: Exprs,
    results: Exprs,
    else_result: Option<Box<Expr>>,
) -> FormaDoc<'a> {
    RcDoc::text("case")
        .append(if let Some(operand) = operand {
            RcDoc::space().append(expr_doc(*operand))
        } else {
            RcDoc::nil()
        })
        .append(
            RcDoc::line().nest(NEST_FACTOR).append(
                RcDoc::intersperse(
                    conditions.iter().zip(results).map(|(condition, result)| {
                        RcDoc::text("when")
                            .append(RcDoc::space())
                            .append(expr_doc(condition.clone()))
                            .append(RcDoc::space())
                            .append(RcDoc::text("then"))
                            .append(RcDoc::space())
                            .append(expr_doc(result))
                    }),
                    RcDoc::line(),
                )
                .append(if let Some(else_result) = else_result {
                    RcDoc::line().nest(NEST_FACTOR).append(
                        RcDoc::text("else")
                            .append(RcDoc::space())
                            .append(expr_doc(*else_result)),
                    )
                } else {
                    RcDoc::nil()
                }),
            ),
        )
        .append(RcDoc::line().append(RcDoc::text("end")))
}

fn cast_doc<'a>(expr: Expr, data_type: DataType) -> FormaDoc<'a> {
    RcDoc::text("cast")
        .append(RcDoc::text("("))
        .append(
            expr_doc(expr)
                .append(RcDoc::space())
                .append(RcDoc::text("as"))
                .append(RcDoc::space())
                .append(RcDoc::text(data_type.to_string().to_lowercase())),
        )
        .append(RcDoc::text(")"))
}

fn collate_doc<'a>(expr: Expr, collation: ObjectName) -> FormaDoc<'a> {
    expr_doc(expr)
        .append(RcDoc::space())
        .append(RcDoc::text("collate"))
        .append(RcDoc::space())
        .append(RcDoc::text(collation.to_string()))
}

fn compound_identifier_doc<'a>(idents: Idents) -> FormaDoc<'a> {
    RcDoc::intersperse(idents.into_iter().map(ident_doc), RcDoc::text("."))
}

fn exists_doc<'a>(query: Query) -> FormaDoc<'a> {
    RcDoc::text("exists").append(RcDoc::softline().append(parenthenized(query_doc(query))))
}

fn extract_doc<'a>(field: DateTimeField, expr: Expr) -> FormaDoc<'a> {
    RcDoc::text("extract")
        .append(RcDoc::text("("))
        .append(
            RcDoc::text(field.to_string().to_lowercase())
                .append(RcDoc::space())
                .append(RcDoc::text("from"))
                .append(RcDoc::space())
                .append(expr_doc(expr)),
        )
        .append(RcDoc::text(")"))
}

fn function_doc<'a>(
    name: ObjectName,
    args: Exprs,
    over: Option<WindowSpec>,
    distinct: bool,
) -> FormaDoc<'a> {
    RcDoc::text(name.to_string().to_lowercase())
        .append(parenthenized(
            if distinct {
                RcDoc::text("distinct").append(RcDoc::space())
            } else {
                RcDoc::nil()
            }
            .append(interweave_comma(args.into_iter().map(expr_doc))),
        ))
        .append(window_spec_doc(over))
}

fn window_spec_doc<'a>(window_spec: Option<WindowSpec>) -> FormaDoc<'a> {
    if let Some(WindowSpec {
        partition_by,
        order_by,
        window_frame,
    }) = window_spec
    {
        RcDoc::space().append(
            RcDoc::text("over").append(parenthenized(
                if !partition_by.is_empty() {
                    RcDoc::text("partition by")
                        .append(RcDoc::space())
                        .append(interweave_comma(partition_by.into_iter().map(expr_doc)))
                        .append(RcDoc::space())
                } else {
                    RcDoc::nil()
                }
                .append(if !order_by.is_empty() {
                    RcDoc::text("order by")
                        .append(RcDoc::space())
                        .append(interweave_comma(order_by.into_iter().map(order_by_doc)))
                } else {
                    RcDoc::nil()
                })
                .append(window_frame_doc(window_frame)),
            )),
        )
    } else {
        RcDoc::nil()
    }
}

fn window_frame_doc<'a>(window_frame: Option<WindowFrame>) -> FormaDoc<'a> {
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
                        .append(RcDoc::text(end_bound.to_string().to_lowercase()))
                } else {
                    RcDoc::nil()
                }),
        )
    } else {
        RcDoc::nil()
    }
}

fn in_list_doc<'a>(expr: Expr, negated: bool, list: Exprs) -> FormaDoc<'a> {
    resolve_negation(expr, negated).append(parenthenized(interweave_comma(
        list.into_iter().map(expr_doc),
    )))
}

fn is_not_null_doc<'a>(expr: Expr) -> FormaDoc<'a> {
    expr_doc(expr)
        .append(RcDoc::space())
        .append(RcDoc::text("is not null"))
}

fn is_null_doc<'a>(expr: Expr) -> FormaDoc<'a> {
    expr_doc(expr)
        .append(RcDoc::space())
        .append(RcDoc::text("is null"))
}

fn in_subquery_doc<'a>(expr: Expr, negated: bool, subquery: Query) -> FormaDoc<'a> {
    resolve_negation(expr, negated).append(parenthenized(query_doc(subquery)))
}

fn listagg_doc<'a>(
    ListAgg {
        distinct,
        expr,
        separator,
        on_overflow,
        within_group,
    }: ListAgg,
) -> FormaDoc<'a> {
    RcDoc::text("listagg")
        .append(parenthenized(
            if distinct {
                RcDoc::text("distinct").append(RcDoc::space())
            } else {
                RcDoc::nil()
            }
            .append(expr_doc(*expr))
            .append(if let Some(separator) = separator {
                RcDoc::text(", ").append(expr_doc(*separator))
            } else {
                RcDoc::nil()
            })
            .append(if let Some(on_overflow) = on_overflow {
                listagg_on_overflow_doc(on_overflow)
            } else {
                RcDoc::nil()
            }),
        ))
        .append(if !within_group.is_empty() {
            RcDoc::line().append(
                RcDoc::text("within group (order by ")
                    .append(interweave_comma(within_group.into_iter().map(order_by_doc)))
                    .append(RcDoc::text(")")),
            )
        } else {
            RcDoc::nil()
        })
}

fn listagg_on_overflow_doc<'a>(on_overflow: ListAggOnOverflow) -> FormaDoc<'a> {
    RcDoc::text(" on overflow").append(match on_overflow {
        ListAggOnOverflow::Error => RcDoc::text(" error"),
        ListAggOnOverflow::Truncate { filler, with_count } => RcDoc::text(" truncate")
            .append(if let Some(filler) = filler {
                RcDoc::space().append(expr_doc(*filler))
            } else {
                RcDoc::nil()
            })
            .append(if with_count {
                RcDoc::text(" with count")
            } else {
                RcDoc::text(" without count")
            }),
    })
}

fn nested_doc<'a>(expr: Expr) -> FormaDoc<'a> {
    RcDoc::text("(")
        .append(RcDoc::softline_())
        .append(expr_doc(expr).group())
        .nest(NEST_FACTOR)
        .append(RcDoc::softline_())
        .append(RcDoc::text(")"))
}

fn qualified_wildcard_doc<'a>(idents: Idents) -> FormaDoc<'a> {
    RcDoc::intersperse(idents.into_iter().map(ident_doc), RcDoc::text("."))
        .append(RcDoc::text(".*"))
}

fn subquery_doc<'a>(query: Query) -> FormaDoc<'a> {
    RcDoc::softline_().append(parenthenized(query_doc(query)))
}

fn typed_string_doc<'a>(data_type: DataType, value: String) -> FormaDoc<'a> {
    RcDoc::text(data_type.to_string().to_lowercase())
        .append(RcDoc::space())
        .append(RcDoc::text(format!("'{}'", value)))
}

fn unary_op_doc<'a>(op: UnaryOperator, expr: Expr) -> FormaDoc<'a> {
    RcDoc::text(op.to_string().to_lowercase()).append(expr_doc(expr))
}

fn value_doc<'a>(value: Value) -> FormaDoc<'a> {
    match value {
        Value::Null => RcDoc::text("null"),
        // TODO: Interval handling does not work for Redshift.
        _ => RcDoc::text(value.to_string()),
    }
}

/// Resolves a possibly negated expression to an `RcDoc`.
pub fn resolve_negation<'a>(expr: Expr, negated: bool) -> FormaDoc<'a> {
    expr_doc(expr)
        .append(RcDoc::space())
        .append(if negated {
            RcDoc::text("not").append(RcDoc::space())
        } else {
            RcDoc::nil()
        })
        .append(RcDoc::text("in"))
        .append(RcDoc::softline())
}

/// Returns `true` if the given `BinaryOperator` should create a newline,
/// otherwise `false`.
pub fn is_newline_op(binop: &BinaryOperator) -> bool {
    *binop == BinaryOperator::And || *binop == BinaryOperator::Or
}
