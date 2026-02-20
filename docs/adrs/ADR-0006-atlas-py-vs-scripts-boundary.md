# ADR-0006: atlas-py And scripts Boundary

- Status: accepted
- Date: 2026-02-20

## Context

The repository now has an internal tooling CLI (`bijux-atlas-scripts`) and needs groundwork for a future user-facing Python library (`bijux-atlas-py`).

## Decision

`bijux-atlas-py` is user-facing API surface, `bijux-atlas-scripts` is internal tooling surface, and `bijux-atlas-py` must not depend on `bijux-atlas-scripts`.

## Consequences

- `bijux-atlas-py` may depend on stable file/data format contracts (sqlite/parquet) and HTTP client behavior.
- Optional future FFI route with `pyo3` is allowed later via a dedicated crate, not in this groundwork change.
- Tooling changes in `bijux-atlas-scripts` are not user API guarantees.
