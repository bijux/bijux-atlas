# Reference Grade Acceptance Checklist

- Owner: `bijux-atlas-product`

## What

Executable release checklist mapped to CI jobs.

## Why

Release quality must be enforced by gates, not narrative claims.

## Scope

Applies before tagging release artifacts.

## Non-goals

No manual-only checklist items without corresponding gate.

## Contracts

- [ ] `dev-fmt`: formatting clean.
- [ ] `dev-lint`: lint clean.
- [ ] `dev-audit`: dependency and security checks clean.
- [ ] `dev-test-all`: all tests pass with no skips in required suites.
- [ ] `dev-coverage`: coverage gate passes.
- [ ] `openapi-drift`: OpenAPI snapshots match.
- [ ] `docs`: docs build + lint + link checks pass.
- [ ] `docs-freeze`: generated docs are up-to-date.
- [ ] `ssot-check`: contracts checks pass.

## Failure modes

Any unchecked item blocks release.

## How to verify

```bash
$ make -j8 dev-fmt dev-lint dev-audit dev-test-all dev-coverage openapi-drift docs docs-freeze ssot-check
```

Expected output: all targets finish successfully.

## See also

- [Contracts SSOT](../contracts/README.md)
- [SLO Targets](SLO_TARGETS.md)
- [Production Readiness](../operations/production-readiness-checklist.md)
