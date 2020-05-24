extern crate formation;

use std::fs;

use pretty_assertions::assert_eq;

const MAX_WIDTH: usize = 100;

#[test]
fn test_format() {
    let sql_string = fs::read_to_string("tests/sql/simple.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/simple_expected.sql").unwrap()]
    );
}

#[test]
fn test_outer_join() {
    let sql_string = fs::read_to_string("tests/sql/outer_join.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/outer_join_expected.sql").unwrap()]
    );
}

#[test]
fn test_inner_join() {
    let sql_string = fs::read_to_string("tests/sql/inner_join.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/inner_join_expected.sql").unwrap()]
    );
}

#[test]
fn test_between() {
    let sql_string = fs::read_to_string("tests/sql/between.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/between_expected.sql").unwrap()]
    );
}

#[test]
fn test_subquery() {
    let sql_string = fs::read_to_string("tests/sql/subquery.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/subquery_expected.sql").unwrap()]
    );
}

#[test]
fn test_correlated_subquery() {
    let sql_string = fs::read_to_string("tests/sql/correlated_subquery.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/correlated_subquery_expected.sql").unwrap()]
    );
}

#[test]
fn test_case() {
    let sql_string = fs::read_to_string("tests/sql/case.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/case_expected.sql").unwrap()]
    );
}

#[test]
fn test_evaluation_order() {
    let sql_string = fs::read_to_string("tests/sql/evaluation_order.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/evaluation_order_expected.sql").unwrap()]
    );
}

#[test]
fn test_window_function() {
    let sql_string = fs::read_to_string("tests/sql/window_function.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/window_function_expected.sql").unwrap()]
    );
}

#[test]
fn test_null() {
    let sql_string = fs::read_to_string("tests/sql/null.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/null_expected.sql").unwrap()]
    );
}

#[test]
fn test_group_by() {
    let sql_string = fs::read_to_string("tests/sql/group_by.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/group_by_expected.sql").unwrap()]
    );
}

#[test]
fn test_natural_join() {
    let sql_string = fs::read_to_string("tests/sql/natural_join.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/natural_join_expected.sql").unwrap()]
    );
}

#[test]
fn test_join_using() {
    let sql_string = fs::read_to_string("tests/sql/join_using.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/join_using_expected.sql").unwrap()]
    );
}
