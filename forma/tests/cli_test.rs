use std::io::{Read, Write};

use assert_cmd::Command;
use tempfile::{self, NamedTempFile};

#[test]
fn cli_no_args() {
    let mut cmd = Command::cargo_bin("forma").unwrap();
    cmd.assert().success();
}

#[test]
fn cli_stdin() {
    let mut cmd = Command::cargo_bin("forma").unwrap();
    let assert = cmd.write_stdin("SELECT * FROM t1").assert();
    assert.success().stdout("select * from t1;\n");
}

#[test]
fn cli_file() {
    let mut example_sql = NamedTempFile::new().unwrap();
    write!(example_sql, "SELECT * FROM t1").unwrap();
    let mut cmd = Command::cargo_bin("forma").unwrap();
    cmd.arg(example_sql.path()).assert().success();
    let mut formatted = String::new();
    example_sql
        .reopen()
        .unwrap()
        .read_to_string(&mut formatted)
        .unwrap();
    assert_eq!(formatted, "select * from t1;\n".to_string());
}

#[test]
fn cli_check() {
    let mut cmd = Command::cargo_bin("forma").unwrap();
    let assert = cmd.write_stdin("SELECT * FROM t1").arg("--check").assert();
    assert.failure().code(1);
}
