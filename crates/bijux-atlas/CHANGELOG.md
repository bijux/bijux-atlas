# Changelog

All notable changes to **bijux-atlas** are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## v0.2.1 – 2026-06-27

### Added
- Reintroduced `bijux-atlas` as a real compatibility alias crate that
  re-exports the canonical `bijux-atlas-runtime` library surface.
- Added alias contract tests so the public `bijux_atlas` import path stays
  equivalent to `bijux_atlas_runtime` for stable modules and functions.
