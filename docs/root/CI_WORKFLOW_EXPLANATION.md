# CI Workflow Explanation

- Owner: `build-and-release`
- Stability: `stable`

CI is lane-based with deterministic reports and artifact uploads.

Primary lanes:
- PR validation
- Docs-only validation
- Ops validation
- Nightly validation
- Release candidate

Reference:
- `docs/development/ci/ci.md`
- `.github/workflows/`
