# Docker Contracts

Policy contracts for docker image governance.

- `base-image-allowlist.json`: approved builder/runtime base images.
- `no-latest.json`: whether `:latest` tags are forbidden.
- `digest-pinning.json`: pinning policy and forbidden latest behavior.
- `sbom-policy.json`: SBOM expectations for release workflows.
- `image-size-budget.json`: runtime image size budget in bytes.
