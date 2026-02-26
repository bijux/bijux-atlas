# Docker Reproducibility

- Owner: docker-governance
- Stability: stable

Container builds are reproducible when lockfiles, base-image pins, and build args are deterministic.

## Pinning policy

- Default rule: base images must use immutable digests (`@sha256:...`).
- Temporary exceptions are tracked in `docker/contracts/digest-pinning.json`.
- `:latest` is forbidden.

## CI evidence

- Runtime image build, SBOM, and vulnerability scan artifacts are uploaded by `ci-pr`.
- Artifacts are published under `artifacts/<run_id>/reports/docker/`.
