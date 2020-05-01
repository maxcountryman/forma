use pretty::RcDoc;
use sqlparser::ast::{BinaryOperator, Expr};
use sqlparser::ast::{Query, Select, SetExpr, Statement};

fn is_newline_op(binop: &BinaryOperator) -> bool {
    let op_string = binop.to_string().to_lowercase();
    op_string == "and" || op_string == "or"
}

fn resolve_negation<'a>(expr: String, negated: bool) -> RcDoc<'a, ()> {
    RcDoc::text(expr)
        .append(RcDoc::space())
        .append(if negated {
            RcDoc::text("not").append(RcDoc::space())
        } else {
            RcDoc::nil()
        })
        .append(RcDoc::text("in"))
        .append(RcDoc::line())
}

fn process_in_expr<'a>(expr: Expr) -> RcDoc<'a, ()> {
    match expr {
        Expr::InSubquery {
            expr,
            negated,
            subquery,
        } => resolve_negation(expr.to_string(), negated)
            .append(
                RcDoc::text("(")
                    .append(RcDoc::line())
                    .append(process_query(*subquery))
                    .nest(2),
            )
            .append(RcDoc::line())
            .append(RcDoc::text(")"))
            .nest(4)
            .group(),
        Expr::InList {
            expr,
            negated,
            list,
        } => resolve_negation(expr.to_string(), negated)
            .append(
                RcDoc::text("(")
                    .append(RcDoc::line())
                    .append(RcDoc::intersperse(
                        list.into_iter().map(|x| x.to_string()),
                        RcDoc::text(",").append(RcDoc::line()),
                    ))
                    .nest(2)
                    .group(),
            )
            .append(RcDoc::line())
            .append(RcDoc::text(")"))
            .nest(4)
            .group(),
        _ => RcDoc::nil(),
    }
}

fn process_expr<'a>(expr: Option<Expr>) -> RcDoc<'a, ()> {
    match expr {
        Some(expr) => match expr {
            Expr::BinaryOp { left, op, right } => process_expr(Some(*left))
                .append(RcDoc::space())
                .append(if is_newline_op(&op) {
                    RcDoc::hardline()
                        .append(RcDoc::text(op.to_string().to_lowercase()))
                        .append(RcDoc::space())
                        .nest(2)
                } else {
                    RcDoc::text(op.to_string()).append(RcDoc::space())
                })
                .append(process_expr(Some(*right))),
            Expr::InSubquery { .. } => process_in_expr(expr),
            Expr::InList { .. } => process_in_expr(expr),
            // TODO: Handle other expression types.
            _ => RcDoc::text(expr.to_string()),
        },
        None => RcDoc::nil(),
    }
}

fn process_query<'a>(query: Query) -> RcDoc<'a, ()> {
    let Query {
        body,
        order_by,
        limit,
        ..
    } = query;
    // TODO: Match body on type, e.g. Select.
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
    doc = if let SetExpr::Select(box Select {
        selection: Some(selection),
        ..
    }) = body
    {
        doc.append(
            RcDoc::line()
                .append(RcDoc::text("where").append(RcDoc::line().nest(2)))
                .append(process_expr(Some(selection))),
        )
    } else {
        doc
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

fn process_statement<'a>(statement: Statement) -> RcDoc<'a, ()> {
    match statement {
        // Select statement.
        Statement::Query(query) => process_query(*query),
        // TODO: Match remaining statement variants.
        _ => unreachable!(),
    }
}

/// Turns normal SQL into delightfully formatted SQL.
///
/// # Examples
///
/// ```
/// use forma::dialect::TemplatedDialect;
/// use forma::doc::prettify_statement;
/// use sqlparser::parser::Parser;
/// let sql = "SELECT * FROM schema.users WHERE created_at > {{date}}";
/// let dialect = TemplatedDialect {};
/// let ast = Parser::parse_sql(&dialect, sql.to_string()).unwrap();
/// assert_eq!(
///     prettify_statement(ast[0].to_owned(), 20),
///     "select\n  *\nfrom\n  schema.users\nwhere\n  created_at > {{date}}"
/// );
/// ```
pub fn prettify_statement(statement: Statement, width: usize) -> String {
    let mut w = Vec::new();
    process_statement(statement).render(width, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}
