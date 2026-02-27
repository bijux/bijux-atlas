# Config Changelog

- Owner: `docs-governance`
- Stability: `stable`

## Purpose

Record breaking and behavior-changing configuration updates with clear migration notes.

## Entries

### 2026-02-27

- Added `atlas-server --validate-config` command to run startup contract checks without serving traffic.
- Added `atlas-server --print-effective-config` command with redacted secrets for deterministic diagnostics.
- Added production guardrails under `ATLAS_ENV=prod`:
  - rejects localhost bind addresses,
  - requires `ATLAS_REDIS_URL`,
  - rejects cached-only mode.

## Policy

- Every breaking config change must add a dated entry and migration note.
- Entry descriptions must reference exact flags/env keys.
