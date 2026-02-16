# Makefiles Public Surface

Stable targets:

- `fmt`
- `lint`
- `check`
- `test` (nextest)
- `test-all` (nextest, includes ignored tests)
- `deny`
- `audit`
- `license-check`
- `policy-lint`
- `docs-check`
- `openapi-drift`
- `build-release`
- `help-smoke`
- `ci`

Developer-loop targets:

- `dev-fmt`
- `dev-lint`
- `dev-check`
- `dev-test`
- `dev-test-all`
- `dev-deny`
- `dev-audit`
- `dev-license-check`
- `dev-policy-lint`
- `dev-ci`
- `dev-clean`

Internal targets are prefixed with `_` and are not stable API.

Related policy docs:

- [Crate layout contract](../crate-layout-contract.md)
