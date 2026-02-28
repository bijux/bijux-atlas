# Control Plane

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@6856280c`
- Reason to exist: define what the control plane owns, forbids, and guarantees.

## Why It Exists

The control plane centralizes operational and governance execution so repository behavior stays deterministic, inspectable, and auditable.

## What It Owns

- Command orchestration for checks, docs, configs, policies, and ops lanes.
- Deterministic report generation and machine-readable artifacts.
- Explicit effect boundaries for Docker, Helm, and environment-dependent workflows.

## Command Families

- `check`
- `docs`
- `configs`
- `ops`
- `policies`

## What It Forbids

- Hidden script entrypoints as primary production workflows.
- Cross-layer shortcut patches that bypass contracts.
- Untracked side effects in CI lanes.

## Exit Codes

- `0`: all selected checks passed
- `1`: one or more non-required checks failed
- `2`: usage error
- `3`: internal runner error
- `4`: one or more required checks failed

## Lanes

- Local lane: fast contributor feedback.
- Pull request lane: required contract and policy gates.
- Merge lane: full required validation with stable outputs.
- Release lane: highest assurance and operator-ready evidence.

## CI Mode

- Use `bijux dev atlas ... --ci` for CI-facing runs.
- CI mode forces CI profile behavior and disables ANSI color output.

## Effect Boundaries

- `ops/` stores operational data, schemas, and runbooks.
- Effectful ops actions run through explicit `ops` surfaces.
- User-facing operations use `bijux dev atlas ops ...` (or thin `make` wrappers), not raw scripts.

## What This Page Is Not

This page is not a command reference and not an incident runbook.

## Example

```bash
bijux dev atlas docs validate --ci
```

## What to Read Next

- Runtime boundaries: [Architecture Boundaries](../architecture/boundaries.md)
- CI behavior: [CI Overview](ci-overview.md)
- Core terms: [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
