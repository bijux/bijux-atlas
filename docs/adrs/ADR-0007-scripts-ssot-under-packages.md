# ADR-0007: Scripts SSOT Under Packages

- Status: Accepted
- Date: 2026-02-20
- Owner: platform

## Context

Repository automation logic was historically spread across `scripts/` and `tools/` paths, which made packaging, dependency management, and CLI surface governance harder to enforce.

## Decision

- `atlasctl` is the SSOT tooling product surface.
- Python package roots are `packages/` (primary) and `tools/` during transition.
- Make targets must invoke package entrypoints, not arbitrary `python <path>` scripts.
- Legacy `scripts/` remains transition-only until complete removal gates are satisfied.

## Consequences

- Lockfile-based Python dependency management is mandatory.
- Command surface and output schemas are testable and versioned.
- CI can enforce deterministic tooling behavior across macOS and Linux containers.
