#![feature(box_syntax, box_patterns)]

use crate::dialect::TemplatedDialect;
use pretty::RcDoc;
use sqlparser::ast::SetExpr;
use sqlparser::ast::{Query, Select, Statement};
use sqlparser::parser::Parser;

mod dialect;

fn to_doc<'a>(statement: Statement) -> RcDoc<'a, ()> {
    match statement {
        // Select statement.
        Statement::Query(box Query {
            body,
            order_by,
            limit,
            ..
        }) => {
            let mut doc: RcDoc<'a, ()> = RcDoc::text("select").append(RcDoc::line());

            doc = if let SetExpr::Select(box Select { projection, .. }) = body.to_owned() {
                doc.nest(2).append(
                    RcDoc::intersperse(
                        projection.into_iter().map(|x| x.to_string()),
                        RcDoc::text(",").append(RcDoc::line()),
                    )
                    .nest(2)
                    .group(),
                )
            } else {
                doc
            };
            doc = if let SetExpr::Select(box Select { from, .. }) = body.to_owned() {
                doc.append(
                    RcDoc::line()
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
            //doc = if let SetExpr::Select()
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
            doc = if let Some(limit) = limit {
                doc.append(
                    RcDoc::line()
                        .append(RcDoc::text("limit").append(RcDoc::line().nest(2)))
                        .append(RcDoc::text(limit.to_string())),
                )
            } else {
                doc
            };
            doc
        }
        // TODO: Match remainer statement variants.
        _ => unreachable!(),
    }
}

fn to_pretty(statement: Statement, width: usize) -> String {
    let mut w = Vec::new();
    to_doc(statement).render(width, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}

fn main() {
    let sql =
        "select id, name, email from {{table_a}}, {{table_b}} where created_at > {{date}} and id in (1, 2, 3) order by name, email limit 3; -- foo";

    let dialect = TemplatedDialect {};
    let ast = Parser::parse_sql(&dialect, sql.to_string()).unwrap();
    for statement in ast {
        println!("{}", to_pretty(statement, 30));
    }
}
