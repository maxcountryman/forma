use pretty::RcDoc;
use sqlparser::ast::{Cte, Expr, Fetch, Offset, OffsetRows, Query};

use crate::constants::NEST_FACTOR;
use crate::doc::body::body_doc;
use crate::doc::common::{interweave_comma, order_by_doc, parenthenized, FormaDoc};
use crate::doc::expr::expr_doc;

/// Transforms the given `Query` into an `RcDoc`.
pub fn query_doc<'a>(
    Query {
        ctes,
        body,
        order_by,
        limit,
        offset,
        fetch,
    }: Query,
) -> FormaDoc<'a> {
    // CTEs.
    ctes_doc(ctes)
        // Query body, e.g. `select * from t1 where x > 1`.
        .append(body_doc(body))
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
        .append(limit_doc(limit))
        // Offset.
        .append(offset_doc(offset))
        // Fetch.
        .append(fetch_doc(fetch))
        .group()
}

fn ctes_doc<'a>(ctes: Vec<Cte>) -> FormaDoc<'a> {
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
}

fn limit_doc<'a>(limit: Option<Expr>) -> FormaDoc<'a> {
    if let Some(limit) = limit {
        RcDoc::line()
            .append(RcDoc::text("limit").append(RcDoc::line().nest(NEST_FACTOR)))
            .append(RcDoc::text(limit.to_string()))
    } else {
        RcDoc::nil()
    }
}

fn offset_doc<'a>(offset: Option<Offset>) -> FormaDoc<'a> {
    if let Some(Offset { value, rows }) = offset {
        RcDoc::line().append(RcDoc::text(format!("offset {}", value)).append(match rows {
            OffsetRows::None => RcDoc::nil(),
            OffsetRows::Row => RcDoc::text(" row"),
            OffsetRows::Rows => RcDoc::text(" rows"),
        }))
    } else {
        RcDoc::nil()
    }
}

fn fetch_doc<'a>(fetch: Option<Fetch>) -> FormaDoc<'a> {
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
    }
}
