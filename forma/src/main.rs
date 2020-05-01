use sqlparser::parser::Parser;

use formation::{prettify_statement, TemplatedDialect};

fn main() {
    let sql = "select * from users where users.id > 100";
    let dialect = TemplatedDialect {};
    let ast = Parser::parse_sql(&dialect, sql.to_string()).unwrap();
    for statement in ast {
        println!("{}", prettify_statement(statement, 35));
    }
}
