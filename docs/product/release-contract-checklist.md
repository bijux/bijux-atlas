# Release Contract Checklist

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

- [ ] `fmt`: formatting clean.
- [ ] `lint`: lint clean.
- [ ] `audit`: dependency and security checks clean.
- [ ] `test`: all required tests pass (use `bijux dev atlas dev test --all` when validating ignored suites).
- [ ] `coverage`: coverage gate passes.
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

- [Governance Index](../governance/index.md)
- [SLO Targets](slo-targets.md)
- [Deploy Workflow](../operations/deploy.md)
