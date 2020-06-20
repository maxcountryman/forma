<h1 align="center">
forma 🐚
</h1>

<p align="center">
An opinionated SQL formatter.
</p>

<div align="center">
<a href="https://github.com/maxcountryman/forma/actions">
<img src="https://github.com/maxcountryman/forma/workflows/Main/badge.svg"/>
</a>
<a href='https://coveralls.io/github/maxcountryman/forma?branch=master'><img src='https://coveralls.io/repos/github/maxcountryman/forma/badge.svg?branch=master' alt='Coverage Status' /></a>
<a href="https://crates.io/crates/forma">
<img src="http://meritbadge.herokuapp.com/forma"/>
</a>
<a href="http://docs.rs/formation">
<img src="https://docs.rs/formation/badge.svg"/>
</a>  
<a href="https://maxcountryman.github.io/forma">
<img src="https://img.shields.io/badge/docs-master-green.svg"/>
</a>
</div>

<br />

## 📦 Install

The binary may be installed via `cargo`:

```
$ cargo install forma
```

Further the companion library `formation` may be required as a dependency:

```toml
[dependencies]
formation = "0.2.0"
```

## 🤸 Usage

> ⚠️ `forma` should be considered alpha quality, with several known, and many
more unknown deficiencies. **Use at your own risk!**

Give `forma` some SQL via a file path or stdin and you'll get back formatted
SQL.

```
$ echo "SELECT * FROM users" | forma
select * from users;
```

To format a SQL file, simply give `forma` the file path.

```
$ forma path/to/some/sql/example.sql
```

And if you'd prefer to not actually format the SQL but know if formatting
would happen then use the `--check` flag.

The binary is a thin wrapper around the `formation` library, which can be used
in your own applications to format SQL.

```rust
use formation::format;
let sql = "SELECT * FROM users;";
assert_eq!(
    format(sql, false, 100).unwrap(),
    vec!["select * from users;\n".to_owned()]
);
```

## 🚧 TODOs

- [ ] Comprehensive `Statement` variant support (currently only `Query`)
- [ ] Support for comments (these will be eaten by the formatter!)
- [ ] Parameterized dialects
