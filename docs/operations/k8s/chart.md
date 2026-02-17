# Helm Chart Contract

- Owner: `bijux-atlas-operations`

## What

Canonical documentation for `charts/bijux-atlas` packaging and deployment semantics.

## Why

Ensures chart behavior is documented in docs spine, not isolated in chart tree.

## Scope

Helm chart templates, values, and deployment profiles.

## Non-goals

Does not duplicate every template file content.

## Contracts

- Chart source remains under `charts/bijux-atlas/`; operations workflows and tests are under `ops/`.
- Values contract: [values.md](values.md)
- Generated values schema: `charts/bijux-atlas/values.schema.json` from `docs/contracts/CHART_VALUES.json`
- Install verification: `ops-k8s-tests`
- Required values keys include `values.server`, `values.store`, `values.cache`, and `values.resources`.

## Failure modes

Undocumented chart changes create deployment regressions.

## How to verify

```bash
$ make ops-values-validate
$ make ops-k8s-template-tests
$ make ops-k8s-tests
```

Expected output: chart schema generation/checks pass and k8s install gates pass.

## See also

- [K8s Index](INDEX.md)
- [Values](values.md)
- [K8s E2E Tests](../e2e/k8s-tests.md)
- `ops-ci`
