# bijux-atlas-py Roadmap

- Owner: `platform`
- Stability: `draft`

## What

Roadmap and non-goals for `bijux-atlas-py`, a future user-facing Python library.

## Roadmap

1. Keep placeholder package importable (`import bijux_atlas_py`).
2. Choose data access route:
   - Stable file/API route (sqlite/parquet + HTTP), or
   - FFI route with dedicated `pyo3` crate (later).
3. Add CI smoke once implementation starts: `python -c "import bijux_atlas_py"`.
4. Publish only after explicit user API contracts exist.

## Non-goals

- No dependency on `bijux-atlas-scripts`.
- No direct reuse of internal tooling command code as library API.
- No FFI crate in this groundwork bundle.

## Boundary With Scripts

- `bijux-atlas-scripts` is an internal operator/developer CLI surface.
- `bijux-atlas-py` is a future user-facing library surface.
- Shared logic must live in stable data contracts or dedicated shared libraries, not in CLI command modules.

## See also

- [bijux-atlas-scripts Tooling](bijux-atlas-scripts.md)
- [ADR-0006 atlas-py vs scripts boundary](../../adrs/ADR-0006-atlas-py-vs-scripts-boundary.md)
