# Architecture

## Purpose

`atlasctl` is the SSOT internal scripting surface for repository gates and report tooling.

## Module Boundaries

- `core`: run context, filesystem policy, process helpers, schema helpers.
- `contracts`: output/schema validation helpers.
- domain modules: `ops`, `docs`, `configs`, `policies`, `make`, `inventory`, `report`, `layout`, `registry`.

Rules:
- Domain modules do not import each other directly unless explicitly allowed.
- Shared utilities belong in `core`.
- All command outputs that are machine-consumed must support deterministic JSON.

## Non-scope

- User-facing Python SDK logic belongs in `packages/bijux-atlas-py`.
- Runtime application logic does not belong in this package.
