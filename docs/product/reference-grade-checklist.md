# Reference Grade Checklist

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
- [ ] `dev-test`: all required tests pass (use `atlasctl dev test --all` when validating ignored suites).
- [ ] `dev-coverage`: coverage gate passes.
- [ ] `openapi-drift`: OpenAPI snapshots match.
- [ ] `docs`: docs build + lint + link checks pass.
- [ ] `docs-freeze`: generated docs are up-to-date.
- [ ] `ssot-check`: contracts checks pass.

## Failure modes

Any unchecked item blocks release.

## How to verify

```bash
$ make ci
```

Expected output: formatting, lint, audit, test, coverage, OpenAPI drift, docs, and SSOT checks finish successfully.

## See also

- [Contracts SSOT](../contracts/INDEX.md)
- [SLO Targets](slo-targets.md)
- [Production Readiness](../operations/production-readiness-checklist.md)
