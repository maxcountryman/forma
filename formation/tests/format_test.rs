extern crate formation;

use std::fs;

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
fn test_between() {
    let sql_string = fs::read_to_string("tests/sql/between.sql").unwrap();
    assert_eq!(
        formation::format(sql_string, false, MAX_WIDTH).unwrap(),
        vec![fs::read_to_string("tests/sql/between_expected.sql").unwrap()]
    );
}
