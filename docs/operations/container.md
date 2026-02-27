# Container and Kubernetes

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## What

This page defines how container images map into Kubernetes chart deployment.

## Image Contract

- Validate policy and static contracts via `make docker-validate`.
- Build image via `make docker-build`.
- Smoke image via `make docker-smoke`.
- Generate SBOM via `make docker-sbom`.
- Scan image via `make docker-scan`.
- Run the full docker lane via `make docker-gate`.
- Release (explicit override) via `make docker-release` (CI only).

## Digest and Pinning

- Kubernetes chart values must reference pinned image tags/digests according to chart policy checks.
- `latest` tags are forbidden in Docker and Helm surfaces.
- Chart/image compatibility is validated by existing ops image pin gates.

## Labels and Provenance

Image must contain OCI labels:
- `org.opencontainers.image.version`
- `org.opencontainers.image.revision`
- `org.opencontainers.image.created`
- `org.opencontainers.image.source`
- `org.opencontainers.image.ref.name`

## How It Relates to Charts

- Docker image provides runtime binary surface.
- Helm chart controls deployment profile, rollout policy, and image pull coordinates.
- Deployment correctness is validated by public gates such as `make ops-k8s-suite` and image pin checks.

## Verify

```bash
make docker-gate
```
