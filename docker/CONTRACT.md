# Docker Contract

- Owner: `bijux-atlas-platform`
- Policy file: `docker/policy.json`

## Rule

Contract has no mapped gate means contract does not exist.

## Contract To Gate Matrix

| Contract ID | Contract | Gate Command |
| --- | --- | --- |
| `DOCKER-001` | Forbid `:latest` and floating base tags | `bijux dev atlas docker validate` |
| `DOCKER-002` | Enforce digest-pinned base images or explicit exceptions | `bijux dev atlas docker validate` |
| `DOCKER-003` | Runtime Dockerfile copy sources must exist | `bijux dev atlas docker validate` |
| `DOCKER-004` | Build network tokens constrained by policy allowlist | `bijux dev atlas docker validate` |
| `DOCKER-005` | Docker policy source is single file `docker/policy.json` | `bijux dev atlas docker validate` |
| `DOCKER-006` | Runtime smoke surface (`--help`, `--version`) | `bijux dev atlas docker smoke --allow-subprocess` |
| `DOCKER-007` | SBOM artifact generation | `bijux dev atlas docker sbom --allow-subprocess` |
| `DOCKER-008` | Vulnerability scan report generation | `bijux dev atlas docker scan --allow-subprocess --allow-network` |

## Directory Contract

- Allowed docs in `docker/`: `README.md`, `CONTRACT.md`
- Additional markdown files under `docker/` are forbidden.
- Policy meaning is enforced by gates, not by prose-only documents.

## Verification

```bash
make docker-validate
make docker-build
make docker-smoke
make docker-sbom
make docker-scan
```
