# Changelog

All notable changes to **bijux-atlas-api** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Clarified `bijux-atlas-api` as the owner of the `bijux-atlas-openapi`
  binary, API client, DTO, parameter, wire, and error-envelope contracts in
  the split Atlas workspace.

### Fixed
- Removed release-doc drift that still pointed API readers at runtime-owned
  paths instead of the API crate's own surfaces.

## v0.2.1 – 2026-06-27

### Added
- Reintroduced a dedicated Atlas API crate for request parsing, response
  contracts, OpenAPI generation, owned API tests, and API benchmark coverage.
