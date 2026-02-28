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

## Minimal recipe

1. Start with `cargo run -q -p bijux-dev-atlas -- check list` to find the closest existing suite and tags.
2. Add the check implementation and make its evidence message actionable in one sentence.
3. Register the check in the smallest suite that preserves the intended lane coverage.
4. Validate locally with a focused command before rerunning the full lane wrapper.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- check list
cargo test -q -p bijux-dev-atlas -- --nocapture
make ci-fast
```

## Next steps

- [Add a gate policy](add-a-gate-policy.md)
- [Debug failing checks](debug-failing-checks.md)
- [Evidence writing style](evidence-writing-style.md)
