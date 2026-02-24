# Required Status Checks

- Owner: `build-and-release`

Required branch-protection checks for `main`:

- `ci-pr / minimal-root-policies`
- `ci-pr / validate-pr`
- `ci-pr / supply-chain`
- `ci-pr / workflow-policy`
- `docs-only / docs`
- `ops-validate / validate`

Optional PR checks:

- `ops-integration-kind / kind-integration` (manual dispatch or nightly)

Nightly required checks:

- `ci-nightly / nightly-validation`
