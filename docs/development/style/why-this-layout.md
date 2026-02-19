# Why This Layout

## Principle
Cap the public surface and keep internals flexible.

## Why
- Fewer stable entrypoints reduce operational ambiguity.
- Internal refactors remain safe when callers depend only on public contracts.
- SSOT-driven docs and checks prevent drift between intent and implementation.

## How
- Keep runnable entrypoints in `ops/run/` and public make targets only.
- Route docs through public make commands, not internal script paths.
- Emit deterministic artifacts and gate reports for reproducibility.
