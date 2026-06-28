# Changelog

All notable changes to **bijux-atlas-api** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.2 – 2026-06-28

### Changed
- Clarified `bijux-atlas-api` as the owner of the `bijux-atlas-openapi`
  binary, API client, DTO, parameter, wire, and error-envelope contracts in
  the split Atlas workspace.
- Expanded release-facing crate documentation so the API surface now describes
  its own ownership boundary instead of reading like a runtime-adjacent helper.

### Fixed
- Removed release-doc drift that still pointed API readers at runtime-owned
  paths instead of the API crate's own surfaces.
- Removed ambiguity between API contract ownership and server process
  ownership.
