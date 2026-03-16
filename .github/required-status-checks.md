# Required Status Checks

- Owner: `build-and-release`
- Source of truth: workflow and job names in `.github/workflows/*.yml`

Required branch-protection checks for `main`:

- `ci-pr / minimal-root-policies`
- `ci-pr / validate-pr`
- `ci-pr / supply-chain`
- `ci-pr / workflow-policy`
- `docs-only / docs`
- `ops-validate / validate`

Optional PR checks:

- `ops-integration-kind / kind-integration` (manual dispatch or nightly)

Nightly health checks:

- `ci-nightly / nightly-validation`

If a workflow name or job name changes, update this file in the same change that renamed it.
