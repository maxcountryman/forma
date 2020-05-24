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
    case(fixture_paths("between")),
    case(fixture_paths("case")),
    case(fixture_paths("correlated_subquery")),
    case(fixture_paths("ctes")),
    case(fixture_paths("evaluation_order")),
    case(fixture_paths("group_by")),
    case(fixture_paths("inner_join")),
    case(fixture_paths("natural_join")),
    case(fixture_paths("null")),
    case(fixture_paths("outer_join")),
    case(fixture_paths("simple")),
    case(fixture_paths("subquery")),
    case(fixture_paths("join_using")),
    case(fixture_paths("values")),
    case(fixture_paths("window_function"))
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
