# Root Contract

## Scope

- Governed surface: `root/` and `CONTRACT.md`.
- SSOT = bijux-dev-atlas contracts runner.
- Effects boundary: this group runs static contracts only.
- Non-goals:
- This document does not replace executable contract checks.
- This document does not grant manual exception authority.

## Lane policy

- `local`: ad hoc developer runs; no merge-blocking selection is implied.
- `pr`: runs all required contracts plus static coverage.
- `merge`: runs required contracts plus effect coverage.
- `release`: runs the full matrix of required, effect, and slow coverage.
- Required contracts artifact: `artifacts/contracts/required.json`.
- Lane guarantees reference: `docs/operations/release/lane-guarantees.md`.

## Contract IDs

| ID | Title | Severity | Type(static/effect) | Enforced by | Artifacts |
| --- | --- | --- | --- | --- | --- |
| `META-REQ-001` | required contracts stay stable and approved | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `META-REQ-002` | required contracts cover every pillar | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `META-REQ-003` | required contracts avoid placeholder stubs | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-001` | repo root matches the sealed surface | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-002` | repo root markdown stays within the documented surface | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-003` | repo root forbids legacy script directories | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-004` | repo root symlinks stay explicitly allowlisted | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-005` | root Dockerfile points at the canonical runtime dockerfile | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-006` | root Makefile stays a thin delegator | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-007` | workspace lockfile stays rooted at the repo root | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-008` | rust toolchain stays pinned at the repo root | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-009` | cargo config avoids implicit network defaults | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-010` | license stays on the approved SPDX track | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-011` | security policy keeps a reporting path | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-012` | contributing guide names the canonical control plane | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-013` | changelog keeps a versioned release header | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-014` | gitignore preserves tracked contract outputs | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-015` | repo root forbids duplicate toolchain authority files | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-016` | repo root keeps a machine-readable surface manifest | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-017` | repo root forbids undeclared binary-like artifacts | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-018` | repo root forbids committed environment files | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-019` | repo root keeps a bounded top-level directory surface | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-020` | repo root manifest keeps single-segment entry paths | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-021` | editorconfig exists for shared formatting contracts | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-022` | repo root keeps a single unambiguous license authority | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-023` | root readme keeps the canonical entrypoint sections | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-024` | root docs avoid legacy control-plane references | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-025` | repo root keeps support routing out of the root surface | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-026` | repo root forbids duplicate policy directories | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-027` | root surface manifest declares the configs and ops ssot roots | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-028` | root surface manifest keeps docs under contract governance | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-029` | repo tree forbids nested git repositories | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-030` | repo root forbids vendor directories and blobs | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-031` | repo root forbids oversized root files | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-032` | configs and ops do not duplicate rust toolchain pins | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-033` | release process authority stays inside docs or ops | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-034` | repo root keeps a single contracts command interface | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-035` | make contract wrappers delegate to the contracts runner | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-036` | docker make wrappers delegate to the contracts runner | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-037` | repo tree forbids editor backups and platform noise | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-038` | gitattributes line ending policy stays consistent when present | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-039` | workspace members match the actual crate surface | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-040` | workspace crates keep canonical naming | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-041` | top-level contract documents follow the canonical executable template | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-042` | contract registries keep unique contract ids and mapped checks | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-043` | contract registries keep check-to-contract mappings non-orphaned | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |
| `ROOT-044` | contracts list and group execution order stay deterministic | `high` | `static` | `bijux dev atlas contracts root` | `artifacts/contracts/root/report.json` |

## Enforcement mapping

| Contract | Command(s) |
| --- | --- |
| `META-REQ-001` | `bijux dev atlas contracts root --mode static` |
| `META-REQ-002` | `bijux dev atlas contracts root --mode static` |
| `META-REQ-003` | `bijux dev atlas contracts root --mode static` |
| `ROOT-001` | `bijux dev atlas contracts root --mode static` |
| `ROOT-002` | `bijux dev atlas contracts root --mode static` |
| `ROOT-003` | `bijux dev atlas contracts root --mode static` |
| `ROOT-004` | `bijux dev atlas contracts root --mode static` |
| `ROOT-005` | `bijux dev atlas contracts root --mode static` |
| `ROOT-006` | `bijux dev atlas contracts root --mode static` |
| `ROOT-007` | `bijux dev atlas contracts root --mode static` |
| `ROOT-008` | `bijux dev atlas contracts root --mode static` |
| `ROOT-009` | `bijux dev atlas contracts root --mode static` |
| `ROOT-010` | `bijux dev atlas contracts root --mode static` |
| `ROOT-011` | `bijux dev atlas contracts root --mode static` |
| `ROOT-012` | `bijux dev atlas contracts root --mode static` |
| `ROOT-013` | `bijux dev atlas contracts root --mode static` |
| `ROOT-014` | `bijux dev atlas contracts root --mode static` |
| `ROOT-015` | `bijux dev atlas contracts root --mode static` |
| `ROOT-016` | `bijux dev atlas contracts root --mode static` |
| `ROOT-017` | `bijux dev atlas contracts root --mode static` |
| `ROOT-018` | `bijux dev atlas contracts root --mode static` |
| `ROOT-019` | `bijux dev atlas contracts root --mode static` |
| `ROOT-020` | `bijux dev atlas contracts root --mode static` |
| `ROOT-021` | `bijux dev atlas contracts root --mode static` |
| `ROOT-022` | `bijux dev atlas contracts root --mode static` |
| `ROOT-023` | `bijux dev atlas contracts root --mode static` |
| `ROOT-024` | `bijux dev atlas contracts root --mode static` |
| `ROOT-025` | `bijux dev atlas contracts root --mode static` |
| `ROOT-026` | `bijux dev atlas contracts root --mode static` |
| `ROOT-027` | `bijux dev atlas contracts root --mode static` |
| `ROOT-028` | `bijux dev atlas contracts root --mode static` |
| `ROOT-029` | `bijux dev atlas contracts root --mode static` |
| `ROOT-030` | `bijux dev atlas contracts root --mode static` |
| `ROOT-031` | `bijux dev atlas contracts root --mode static` |
| `ROOT-032` | `bijux dev atlas contracts root --mode static` |
| `ROOT-033` | `bijux dev atlas contracts root --mode static` |
| `ROOT-034` | `bijux dev atlas contracts root --mode static` |
| `ROOT-035` | `bijux dev atlas contracts root --mode static` |
| `ROOT-036` | `bijux dev atlas contracts root --mode static` |
| `ROOT-037` | `bijux dev atlas contracts root --mode static` |
| `ROOT-038` | `bijux dev atlas contracts root --mode static` |
| `ROOT-039` | `bijux dev atlas contracts root --mode static` |
| `ROOT-040` | `bijux dev atlas contracts root --mode static` |
| `ROOT-041` | `bijux dev atlas contracts root --mode static` |
| `ROOT-042` | `bijux dev atlas contracts root --mode static` |
| `ROOT-043` | `bijux dev atlas contracts root --mode static` |
| `ROOT-044` | `bijux dev atlas contracts root --mode static` |

## Output artifacts

- `artifacts/contracts/root/report.json`
- `artifacts/contracts/root/registry-snapshot.json`

## Contract to Gate mapping

- Gate: `contracts::root`
- Aggregate gate: `contracts::all`

## Exceptions policy

- No exceptions are allowed by this document.
