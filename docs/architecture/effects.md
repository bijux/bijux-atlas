# Effects

Owner: `architecture`  
Type: `concept`  
Reason to exist: define where side effects are allowed and where purity is required.

## Effect Policy

- Pure crates: `core`, `model`, `ingest`, and pure planning parts of `query`.
- Effectful crates: `store` (storage I/O), `server` (runtime wiring and controlled I/O).
- `api` remains a read-mapping layer and does not execute ingest or mutation effects.

## Scripting And Control Rules

- Repository automation entrypoints are control-plane commands, not direct script execution surfaces.
- Cross-layer fixups are forbidden; defects are fixed in the owning layer.
- Escape hatches require explicit policy approval and ownership.

## Forbidden Patterns

- External process spawning from pure crates.
- Direct filesystem or network side effects from pure planning modules.
- Runtime mutation during report generation.

## Operational Relevance

Effect boundary discipline prevents hidden writes, keeps incidents diagnosable, and preserves deterministic behavior.
