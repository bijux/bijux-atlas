# Add a check in 30 minutes

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: provide a fast path for adding one new control-plane check safely.

## Workflow

1. Define check intent and owner.
2. Implement check logic with deterministic output ordering.
3. Register check in the appropriate suite.
4. Add docs and report coverage references.

## Verify success

```bash
cargo test -q -p bijux-dev-atlas -- --nocapture
```

## Next steps

- [Add a gate policy](add-a-gate-policy.md)
- [Debug failing checks](debug-failing-checks.md)
