extern crate formation;

use std::fs;

use rstest::rstest;

use pretty_assertions::assert_eq;

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
    case::correlated_subquery(fixture_paths("correlated_subquery")),
    case::ctes(fixture_paths("ctes")),
    case::evaluation_order(fixture_paths("evaluation_order")),
    case::group_by(fixture_paths("group_by")),
    case::inner_join(fixture_paths("inner_join")),
    case::join_using(fixture_paths("join_using")),
    case::natural_join(fixture_paths("natural_join")),
    case::null(fixture_paths("null")),
    case::outer_join(fixture_paths("outer_join")),
    case::simple(fixture_paths("simple")),
    case::subquery(fixture_paths("subquery")),
    case::unary(fixture_paths("unary")),
    case::values(fixture_paths("values")),
    case::window_function(fixture_paths("window_function"))
)]
fn test_format(fixture_paths: (String, String)) {
    let (input_path, expected_path) = fixture_paths;
    let sql_string = fs::read_to_string(input_path.clone())
        .unwrap_or_else(|_| panic!("Could not load fixture input path: {}", input_path));
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string(expected_path.clone())
            .unwrap_or_else(|_| panic!("Could not load fixture expected path: {}", expected_path))]
    );
}
