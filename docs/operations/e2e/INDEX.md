# E2E Operations Index

- Owner: `bijux-atlas-operations`

## What

Canonical entrypoint for local and Kubernetes end-to-end validation workflows.

## Why

Keeps e2e behavior, scripts, and drills documented in the docs spine.

## Scope

Stack bootstrap, K8s suites, realdata drills, fixtures, and e2e helper scripts.

## Non-goals

Does not duplicate individual script source files.

## Contracts

- [Stack](stack.md)
- [K8s Tests](k8s-tests.md)
- [Realdata Drills](realdata-drills.md)
- [Fixtures](fixtures.md)
- [Scripts](scripts.md)
- scenario:smoke
- scenario:k8s-suite
- scenario:realdata
- scenario:perf-e2e

## Failure modes

Untracked e2e workflows drift from actual scripts and CI behavior.

## How to verify

```bash
$ make e2e-local
$ make e2e-k8s-suite
```

Expected output: end-to-end scripts complete successfully.

## See also

- [Operations Index](../INDEX.md)
- [Load Index](../load/INDEX.md)
- [Terms Glossary](../../_style/terms-glossary.md)
- `ops-ci`
