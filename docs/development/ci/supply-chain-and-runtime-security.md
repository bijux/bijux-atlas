# Supply Chain and Runtime Security Policy

## SBOM

- Generate SBOM for each released image (SPDX JSON).
- Attach SBOM to release artifacts.

## Image Signing

- Sign published container images using cosign.
- Verification is required before production promotion.

## Runtime Security Scanning

- CI runs Trivy filesystem scan on repository contents.
- CI runs Trivy image scan on built container image.
- Block release on critical vulnerabilities unless explicitly waived.

## Locking and Reproducibility

- Build with `--locked` Cargo resolution.
- Use immutable release tags for container and chart artifacts.
