# Make Target Upgrade Guide

Use this table to migrate renamed or aliased make targets.

| Old Target | New Target | Status |
|---|---|---|
| `local` | `quick` | `deprecated` |
| `local-full` | `local/all` | `deprecated` |
| `contracts` | `policies/all` | `deprecated` |
| `hygiene` | `scripts/all` | `deprecated` |
| `config-validate` | `configs/all` | `deprecated` |
| `ci` | `ci/all` | `deprecated` |
| `nightly` | `nightly/all` | `deprecated` |
| `ops-e2e-smoke` | `ops-e2e --suite smoke` | `alias` |
| `ops-obs-validate` | `ops-observability-validate` | `alias` |
| `ops-upgrade-drill` | `ops-drill-upgrade` | `alias` |
| `ops-rollback-drill` | `ops-drill-rollback` | `alias` |
