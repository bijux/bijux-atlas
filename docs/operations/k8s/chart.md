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

- Chart source: `charts/bijux-atlas/`
- Values contract: [values.md](values.md)
- Install verification: `e2e/k8s/tests/test_install.sh`
- Required values keys include `values.server`, `values.store`, `values.cache`, and `values.resources`.

## Failure modes

Undocumented chart changes create deployment regressions.

## How to verify

```bash
$ helm lint charts/bijux-atlas
$ ./e2e/k8s/tests/test_install.sh
```

Expected output: chart lint and install gate pass.

## See also

- [K8s Index](INDEX.md)
- [Values](values.md)
- [K8s E2E Tests](../e2e/k8s-tests.md)
