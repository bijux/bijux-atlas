# Dependency Risk Scoring

Atlas computes dependency risk posture from vulnerability counts, source trust, and policy exceptions.

Scoring inputs:

- vulnerability summary totals (`critical`, `high`, `medium`, `low`)
- vulnerability budget and approved, non-expired overrides
- lockfile completeness and source allowlist conformance
- action pinning and container image digest/SBOM evidence

Pass condition:

- release candidate remains within approved risk budget and has no unresolved policy gaps.

Artifacts:

- `artifacts/security/security-vulnerability-scan.json`
- `artifacts/security/dependency-inventory.json`
- `artifacts/security/security-github-actions.json`
