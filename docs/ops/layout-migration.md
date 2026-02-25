# Ops Layout Migration

Policy decisions:

- `ops/_artifacts/`: deleted for local-only transient output. Use `artifacts/` or `ops/_generated/` (non-tracked runtime output) instead.
- `ops/_generated/`: runtime-generated and never hand-edited. Do not commit files here.
