# Dependency Security Scanning

Atlas enforces dependency security with four controls:

1. Registry and index allowlists for Rust, npm, Python, and Helm sources.
2. Mandatory lockfiles and dependency inventory generation.
3. GitHub Actions pinning to full commit SHAs with bounded exceptions.
4. Container base image digest and SBOM coverage verification.

Primary validation command:

- `bijux-dev-atlas security validate`
