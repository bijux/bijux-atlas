# Governance Suites

These files are the machine-readable SSOT for suite inventory ownership.

- `checks.suite.json` lists governed non-test quality gates.
- `contracts.suite.json` lists governed invariants enforced as contracts.
- `tests.suite.json` reserves the stable suite id for future test-registry adoption.
- `default-jobs.json` defines the governed auto-concurrency policy per suite.
- `baseline.json` records the minimum accepted suite inventory so accidental shrink is rejected.

Owner: `team:atlas-governance`
