# CI Config SSOT

This directory is the source of truth for CI-specific config contracts.

- `env-contract.json`: required environment keys for workflow jobs.
- `lanes.json`: canonical CI lane definitions and workflow mapping.

Workflows in `.github/workflows` must stay aligned with these files.
