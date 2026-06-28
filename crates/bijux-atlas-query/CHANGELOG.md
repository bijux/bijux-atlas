# Changelog

All notable changes to **bijux-atlas-query** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Clarified `bijux-atlas-query` as the owner of parsing, planning, cursor, and
  SQLite execution behavior exposed through Atlas runtime surfaces.

### Fixed
- Removed release-doc drift that still described query ownership through the
  old pre-split runtime tree.

## v0.2.1 – 2026-06-27

### Added
- Reintroduced a dedicated Atlas query crate for parsing, planning, SQLite
  execution, cursoring, owned fixture contracts, and query-local benchmarks.
