# Required Status Checks

- Owner: `build-and-release`

Required branch-protection checks for `main`:

- `workflows-make-only`
- `layout-check`
- `script-entrypoints`
- `fmt`
- `clippy`
- `test-nextest`
- `deny`
- `audit`
- `license-check`
- `policy-lint`
- `policy-schema-drift`
- `config-check`
- `ssot-drift`
- `crate-structure`
- `crate-docs-contract`
- `cli-command-surface`
- `docs-check`
- `openapi-drift`
- `query-plan-gate`
- `compatibility-matrix-validate`

Optional PR checks:

- `ops-smoke` (from `ops-smoke-pr` workflow, `continue-on-error`)
- `k8s-pr-subset` (from `k8s-e2e-pr`; includes boundary lint + stack smoke for ops/e2e changes)

Nightly required checks:

- `ops-full` (from `ops-full-nightly`)
- `ops-load-full` (from `ops-full-nightly`)
- `ops-k8s-tests` (from `ops-full-nightly`)
