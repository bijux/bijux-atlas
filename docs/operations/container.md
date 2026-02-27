# Container and Kubernetes

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## What

This page defines how container images map into Kubernetes chart deployment.

## Image Contract

- Validate static contracts via `make docker-contracts`.
- Run effect contracts (build/smoke/sbom/scan) via `make docker-contracts-effect`.
- Run the default docker lane via `make docker-gate`.

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

Related contracts: OPS-ROOT-023, OPS-ROOT-017.
