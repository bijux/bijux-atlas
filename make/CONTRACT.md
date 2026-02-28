# Make Contract

## Scope

- Governed surface: `make/` and `make/CONTRACT.md`.
- SSOT = bijux-dev-atlas contracts runner.
- Effects boundary: this group runs static contracts only.
- Non-goals:
- This document does not replace executable contract checks.
- This document does not grant manual exception authority.

## Contract IDs

| ID | Title | Severity | Type(static/effect) | Enforced by | Artifacts |
| --- | --- | --- | --- | --- | --- |
| `MAKE-ART-001` | make run scoped artifacts | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-CI-001` | make workflow curated usage | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-DIR-001` | make root docs boundary | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-DIR-002` | make nested docs removal | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-DIR-003` | make root file boundary | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-DOCKER-001` | make docker contract boundary | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-DOCS-001` | make docs line budgets | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-DRIFT-001` | make target list drift | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-ENGINE-001` | make direct tool boundary | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-ENV-001` | make env file singularity | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-ENV-002` | make env role boundary | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-GATES-001` | make gate mapping coverage | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-GATES-002` | make public gate visibility | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-GATES-003` | make orphan public gate removal | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-INCLUDE-001` | make root include entrypoint | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-INCLUDE-002` | make public include boundary | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-INCLUDE-003` | make include graph acyclic | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-INTERNAL-001` | make internal target prefix | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-INTERNAL-002` | make scripts banned | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-NAME-001` | make helper file naming | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-NAME-002` | make public file naming | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-NET-001` | make network commands banned | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-OPS-001` | make ops control plane boundary | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-REPRO-001` | make runenv exports | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-SHELL-001` | make shell path stability | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-SHELL-002` | make shell pipeline bound | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-SSOT-001` | make contracts authority | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-STRUCT-002` | make wrapper files only | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-SURFACE-001` | make curated source of truth | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-SURFACE-002` | make curated target budget | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-SURFACE-003` | make curated registry sync | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-SURFACE-005` | make delegate only wrappers | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |
| `MAKE-TARGETLIST-001` | make target list policy | `high` | `static` | `bijux dev atlas contracts make` | `artifacts/contracts/make/report.json` |

## Enforcement mapping

| Contract | Command(s) |
| --- | --- |
| `MAKE-ART-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-CI-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-DIR-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-DIR-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-DIR-003` | `bijux dev atlas contracts make --mode static` |
| `MAKE-DOCKER-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-DOCS-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-DRIFT-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-ENGINE-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-ENV-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-ENV-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-GATES-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-GATES-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-GATES-003` | `bijux dev atlas contracts make --mode static` |
| `MAKE-INCLUDE-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-INCLUDE-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-INCLUDE-003` | `bijux dev atlas contracts make --mode static` |
| `MAKE-INTERNAL-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-INTERNAL-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-NAME-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-NAME-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-NET-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-OPS-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-REPRO-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-SHELL-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-SHELL-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-SSOT-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-STRUCT-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-SURFACE-001` | `bijux dev atlas contracts make --mode static` |
| `MAKE-SURFACE-002` | `bijux dev atlas contracts make --mode static` |
| `MAKE-SURFACE-003` | `bijux dev atlas contracts make --mode static` |
| `MAKE-SURFACE-005` | `bijux dev atlas contracts make --mode static` |
| `MAKE-TARGETLIST-001` | `bijux dev atlas contracts make --mode static` |

## Output artifacts

- `artifacts/contracts/make/report.json`
- `artifacts/contracts/make/registry-snapshot.json`

## Contract to Gate mapping

- Gate: `contracts::make`
- Aggregate gate: `contracts::all`

## Exceptions policy

- No exceptions are allowed by this document.
