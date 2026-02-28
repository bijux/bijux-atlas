# Configs Contract

## Scope

- Governed surface: `configs/` and `configs/CONTRACT.md`.
- SSOT = bijux-dev-atlas contracts runner.
- Effects boundary: this group runs static contracts only.
- Non-goals:
- This document does not replace executable contract checks.
- This document does not grant manual exception authority.

## Contract IDs

| ID | Title | Severity | Type(static/effect) | Enforced by | Artifacts |
| --- | --- | --- | --- | --- | --- |
| `CONFIGS-001` | configs root keeps only declared root files | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-002` | configs files are documented by the registry | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-003` | configs path depth stays within budget | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-004` | configs internal surfaces stay explicitly internal | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-005` | configs groups declare owners | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-006` | configs groups declare schema coverage | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-007` | configs lockfile pairs stay complete | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-008` | configs registry avoids duplicate group ownership | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-009` | generated config surfaces stay separate from authored files | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-010` | configs contracts doc mirrors executable checks | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-011` | configs registry keeps a complete root surface | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-012` | configs registry leaves no orphan files | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-013` | configs registry leaves no dead entries | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-014` | configs group count stays within budget | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-015` | configs group paths stay within group depth budget | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-016` | configs files declare exactly one visibility class | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-017` | configs groups declare tool entrypoints | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-018` | configs groups declare schema ownership | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-019` | configs groups declare lifecycle metadata | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-020` | configs generated index stays deterministic | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-021` | configs generated index matches committed output | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-022` | configs json surfaces parse cleanly | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-023` | configs yaml surfaces parse cleanly | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-024` | configs toml surfaces parse cleanly | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-025` | configs text surfaces avoid whitespace drift | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-026` | configs docs directory forbids nested markdown | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-027` | configs docs directory stays tooling only | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-028` | configs owner map stays aligned with the registry | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-029` | configs consumer map stays aligned with the registry | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-030` | configs public files declare file-level consumers | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-031` | configs json files declare file-level schema coverage | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-032` | configs root json surfaces stay canonical | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-033` | configs schema index matches committed output | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-034` | configs input schemas stay referenced | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-035` | configs schema versioning policy stays complete | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-036` | configs exclusions carry approval and expiry metadata | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |
| `CONFIGS-037` | configs surfaces forbid mutable latest-tag references | `high` | `static` | `bijux dev atlas contracts configs` | `artifacts/contracts/configs/report.json` |

## Enforcement mapping

| Contract | Command(s) |
| --- | --- |
| `CONFIGS-001` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-002` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-003` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-004` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-005` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-006` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-007` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-008` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-009` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-010` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-011` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-012` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-013` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-014` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-015` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-016` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-017` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-018` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-019` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-020` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-021` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-022` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-023` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-024` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-025` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-026` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-027` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-028` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-029` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-030` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-031` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-032` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-033` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-034` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-035` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-036` | `bijux dev atlas contracts configs --mode static` |
| `CONFIGS-037` | `bijux dev atlas contracts configs --mode static` |

## Output artifacts

- `artifacts/contracts/configs/report.json`
- `artifacts/contracts/configs/registry-snapshot.json`

## Contract to Gate mapping

- Gate: `contracts::configs`
- Aggregate gate: `contracts::all`

## Exceptions policy

- No exceptions are allowed by this document.
