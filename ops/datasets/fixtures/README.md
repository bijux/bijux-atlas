# Fixtures Module

Purpose: provide versioned, reusable fixture assets used by ops datasets, load, and e2e suites.

Entry points:
- `make ops-datasets-verify`
- `make ops-e2e-smoke`

Contracts:
- `ops/datasets/fixtures/CONTRACT.md`

Artifacts:
- Fixture assets are inputs; generated artifacts are written under `artifacts/atlas-dev/...`.

Failure modes:
- Missing fixture version directory or lock file.
- Fixture payload drift without contract/schema update.
