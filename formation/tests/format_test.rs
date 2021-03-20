use std::fs;
use std::iter::FromIterator;

use pretty_assertions::assert_eq;
use rstest::rstest;

use formation::error;

const MAX_WIDTH: usize = 100;

fn fixture_paths(name: &str) -> (String, String) {
    (
        format!("tests/sql/{}.sql", name),
        format!("tests/sql/{}_expected.sql", name),
    )
}

#[rstest(
    fixture_paths,
    case::between(fixture_paths("between")),
    case::case(fixture_paths("case")),
    case::cast(fixture_paths("cast")),
    case::collate(fixture_paths("collate")),
    case::correlated_subquery(fixture_paths("correlated_subquery")),
    case::cross_join(fixture_paths("cross_join")),
    case::ctes(fixture_paths("ctes")),
    case::date(fixture_paths("date")),
    case::evaluation_order(fixture_paths("evaluation_order")),
    case::exists(fixture_paths("exists")),
    case::extract(fixture_paths("extract")),
    case::fetch(fixture_paths("fetch")),
    case::full_join(fixture_paths("full_join")),
    case::function(fixture_paths("function")),
    case::group_by(fixture_paths("group_by")),
    case::having(fixture_paths("having")),
    case::inner_join(fixture_paths("inner_join")),
    case::interval(fixture_paths("interval")),
    case::join_using(fixture_paths("join_using")),
    case::listagg(fixture_paths("listagg")),
    case::natural_join(fixture_paths("natural_join")),
    case::nested(fixture_paths("nested")),
    case::not_null(fixture_paths("not_null")),
    case::null(fixture_paths("null")),
    case::order_by(fixture_paths("order_by")),
    case::outer_join(fixture_paths("outer_join")),
    case::right_join(fixture_paths("right_join")),
    case::simple(fixture_paths("simple")),
    case::subquery(fixture_paths("subquery")),
    case::time(fixture_paths("time")),
    case::timestamp(fixture_paths("timestamp")),
    case::top(fixture_paths("top")),
    case::unary(fixture_paths("unary")),
    case::values(fixture_paths("values")),
    case::window_function(fixture_paths("window_function"))
)]
fn test_format(fixture_paths: (String, String)) -> error::Result<()> {
    let (input_path, expected_path) = fixture_paths;
    let sql_string = fs::read_to_string(input_path)?;
    assert_eq!(
        String::from_iter(formation::format(&sql_string, false, MAX_WIDTH)?),
        fs::read_to_string(expected_path)?
    );
    Ok(())
}
