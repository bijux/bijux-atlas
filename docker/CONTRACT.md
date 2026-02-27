# Docker Contract

- Owner: `bijux-atlas-platform`
- Policy file: `docker/policy.json`

## Rule

Contract has no mapped gate means contract does not exist.

## Contract To Gate Matrix

| Contract ID | Contract | Gate Command |
| --- | --- | --- |
| `DOCKER-001` | Forbid `:latest` base tags | `docker.contract.no_latest` via `bijux dev atlas docker validate` |
| `DOCKER-002` | Enforce digest-pinned base images or explicit exceptions | `docker.contract.digest_pins` via `bijux dev atlas docker validate` |
| `DOCKER-003` | Root `Dockerfile` must be a symlink shim | `docker.contract.root_symlink` via `bijux dev atlas docker validate` |
| `DOCKER-004` | Dockerfile paths and copy/build instruction scope rules | `docker.contract.path_scope` via `bijux dev atlas docker validate` |
| `DOCKER-005` | Required OCI labels must exist | `docker.contract.oci_labels` via `bijux dev atlas docker validate` |
| `DOCKER-006` | Required build args must provide defaults | `docker.contract.build_args` via `bijux dev atlas docker validate` |
| `DOCKER-007` | Runtime smoke surface (`--help`, `--version`) | `docker.contract.runtime_smoke` via `bijux dev atlas docker smoke --allow-subprocess` |
| `DOCKER-008` | SBOM artifact generation | `docker.contract.sbom_generated` via `bijux dev atlas docker sbom --allow-subprocess` |
| `DOCKER-009` | Vulnerability scan report generation | `docker.contract.vuln_scan` via `bijux dev atlas docker scan --allow-subprocess --allow-network` |
| `DOCKER-010` | Runtime image size budget gate | `docker.contract.image_size` via `bijux dev atlas docker build --allow-subprocess` |

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
bijux dev atlas docker contracts --format json
bijux dev atlas docker gates --format json
```
