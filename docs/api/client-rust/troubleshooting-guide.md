# Rust Client Troubleshooting Guide

- Owner: `api-contracts`
- Type: `runbook`
- Audience: `user`
- Stability: `stable`

## Common Issues

- Invalid `base_url` configuration.
- Timeout failures from unreachable runtime.
- Response decode failures due to contract drift.

## Diagnostics

1. Validate config using `ClientConfig::validate`.
2. Run SDK examples against target runtime.
3. Verify runtime compatibility matrix contract.
