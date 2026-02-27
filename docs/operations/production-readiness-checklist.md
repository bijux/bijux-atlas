# Production Readiness Checklist

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `operations`

## What

Checklist of enforced gates for production promotion.

## Why

Maps readiness to executable make targets and CI gates.

## Contracts

- All checks run through make targets.
- No direct script entrypoints in docs.

## Failure modes

Promotion must stop if any required target returns non-zero.

## How to verify

```bash
$ make fmt lint test
$ make audit
$ make ops-full-pr
$ make ops-openapi-validate ops-values-validate ops-observability-validate
```

Expected output: all checks pass with no errors.

## Checklist

- Formatting, lint, audit, and tests are green.
- OpenAPI, chart values, and observability contracts are green.
- `ops-full-pr` passes on PR validation.
- `ops-full-nightly` passes in nightly pipeline.

## See also

- [Full Stack Locally](./full-stack-local.md)
- [K8s Test Contract](./k8s/k8s-test-contract.md)
- [Observability Acceptance Gates](./observability/acceptance-gates.md)

Related contracts: OPS-ROOT-023, OPS-ROOT-017.
