# Changelog

All notable changes to **bijux-atlas-core** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.1 – 2026-06-27

### Added
- Reintroduced a dedicated Atlas core crate for canonical hashing, stable JSON,
  sorting primitives, and shared error-code definitions.
- Added canonical cursor-payload encoding and decoding helpers so runtime
  compatibility layers no longer own core serialization behavior.
