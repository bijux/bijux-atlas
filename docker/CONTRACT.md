# Docker Contract

## Scope

- Governed surface: `docker/` and `docker/CONTRACT.md`.
- SSOT = bijux-dev-atlas contracts runner.
- Effects boundary: effect contracts require explicit runtime opt-in flags.
- Non-goals:
- This document does not replace executable contract checks.
- This document does not grant manual exception authority.

## Contract IDs

| ID | Title | Severity | Type(static/effect) | Enforced by | Artifacts |
| --- | --- | --- | --- | --- | --- |
| `DOCKER-000` | docker directory contract | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-003` | root Dockerfile policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-004` | dockerfile location policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-006` | forbidden tags policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-007` | digest pinning policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-008` | required labels policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-009` | build args defaults policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-010` | forbidden pattern policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-011` | copy source policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-012` | required images exist | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-013` | forbidden extra images | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-014` | branch-like tags forbidden | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-015` | base image allowlist | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-016` | base image lock digest | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-017` | from arg defaults | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-018` | from platform override | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-019` | shell instruction policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-020` | package manager cleanup | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-021` | runtime non-root user | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-022` | final stage user declaration | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-023` | final stage workdir | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-024` | final stage process entry | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-025` | release labels contract | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-026` | secret copy guard | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-027` | add instruction forbidden | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-028` | multistage build required | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-029` | dockerignore required entries | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-030` | reproducible build args | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-031` | final stage network isolation | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-032` | final stage package manager isolation | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-033` | image smoke manifest | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-034` | image manifest schema | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-035` | image manifest completeness | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-036` | image build matrix | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-037` | manifest image builds | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-038` | manifest image smoke | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-039` | ci build pull policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-040` | build metadata artifact | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-041` | manifest sbom coverage | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-042` | scan artifact threshold | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-043` | vulnerability allowlist discipline | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-044` | pip install hash pinning | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-045` | cargo install version pinning | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-046` | go install latest forbidden | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-047` | docker markdown boundary | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-048` | contract document generation | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-049` | contract registry export | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-050` | contract gate map export | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-051` | exceptions registry schema | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-052` | exceptions minimal entries | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-053` | scan profile policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-054` | runtime engine policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-055` | airgap build policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-056` | multi-registry push policy | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-057` | downloaded asset digest pinning | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-058` | vendored binary declaration | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-059` | curl pipe shell forbidden | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-060` | dockerfile formatting | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-061` | canonical config copy paths | `high` | `static` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-100` | build succeeds | `high` | `effect` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-101` | runtime smoke checks | `high` | `effect` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-102` | sbom generated | `high` | `effect` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |
| `DOCKER-103` | scan passes policy | `high` | `effect` | `bijux dev atlas contracts docker` | `artifacts/contracts/docker/report.json` |

## Enforcement mapping

| Contract | Command(s) |
| --- | --- |
| `DOCKER-000` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-003` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-004` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-006` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-007` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-008` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-009` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-010` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-011` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-012` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-013` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-014` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-015` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-016` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-017` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-018` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-019` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-020` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-021` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-022` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-023` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-024` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-025` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-026` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-027` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-028` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-029` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-030` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-031` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-032` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-033` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-034` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-035` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-036` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-037` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-038` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-039` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-040` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-041` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-042` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-043` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-044` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-045` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-046` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-047` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-048` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-049` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-050` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-051` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-052` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-053` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-054` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-055` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-056` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-057` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-058` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-059` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-060` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-061` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-100` | `bijux dev atlas contracts docker --mode effect` |
| `DOCKER-101` | `bijux dev atlas contracts docker --mode effect` |
| `DOCKER-102` | `bijux dev atlas contracts docker --mode effect` |
| `DOCKER-103` | `bijux dev atlas contracts docker --mode effect` |

## Output artifacts

- `artifacts/contracts/docker/report.json`
- `artifacts/contracts/docker/registry-snapshot.json`

## Contract to Gate mapping

- Gate: `contracts::docker`
- Aggregate gate: `contracts::all`

## Exceptions policy

- No exceptions are allowed by this document.
