# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2020-06-26

### Changed

- Updated `sqlparser-rs` to version `0.5.1` containing fixes for Postgres,
  e.g. better handling of intervals and dates.
- Provided explicit support for `INTERVAL` formatting.

## [0.2.0] - 2020-06-07

### Changed

- Ensure `field` is lower cased in `EXTRACT` handling.
- Use `&str` instead of `String` which avoids unnecessary copying.
  (Thanks [@Dandandan](https://github.com/Dandandan)!)
- All use of `hardline` has been removed and vertical space will be more
  aggressively conserved.
- The `FormaError` type now uses `thiserror` and `forma` consumes this via
  `anyhow` for significantly more useful error messages.

## [0.1.2] - 2020-05-31

### Added

- Explicit handling of `Value` enum to ensure expected formatting.

### Fixed

- Removed space before unary operators.
- Missing space between `AS` in `CAST`.

## [0.1.1] - 2020-05-29

### Added

- This CHANGELOG.

### Changed

- `JOIN` statements will now wrap on `ON` or `USING` when necessary.

## [0.1.0] - 2020-05-25

### Added

- Initial release: comprehensive formatting for `Query` nodes.

[unreleased]: https://github.com/maxcountryman/forma/compare/0.3.0...HEAD
[0.3.0]: https://github.com/maxcountryman/forma/releases/tag/0.3.0
[0.2.0]: https://github.com/maxcountryman/forma/releases/tag/0.2.0
[0.1.2]: https://github.com/maxcountryman/forma/releases/tag/0.1.2
[0.1.1]: https://github.com/maxcountryman/forma/releases/tag/0.1.1
[0.1.0]: https://github.com/maxcountryman/forma/releases/tag/0.1.0
