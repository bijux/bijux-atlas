# API Troubleshooting Guide

- Owner: `api-contracts`
- Type: `runbook`
- Audience: `user`
- Stability: `stable`

## Common issues

- Unexpected response schema.
- Endpoint not found.
- Version mismatch in OpenAPI contract checks.

## Verification sequence

1. Run `bijux-dev-atlas api verify --format json`.
2. Run `bijux-dev-atlas api diff --format json`.
3. Run `bijux-dev-atlas api validate --format json`.
