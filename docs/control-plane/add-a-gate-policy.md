# Add a gate policy

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: define how to add new gating policy without weakening deterministic enforcement.

## Workflow

1. Define policy intent, scope, and failure condition.
2. Add deterministic policy evaluation and report fields.
3. Add pass/fail tests and remediation guidance.
4. Wire policy into lane mapping.

## Example policy shapes

- hard fail: missing required ownership or schema field
- soft fail promoted later: noisy or expensive checks that need evidence first
- lane-scoped fail: release-only or nightly-only enforcement where local cost is too high

## Verify success

```bash
cargo test -q -p bijux-dev-atlas -- --nocapture
make ci-pr
```

## Next steps

- [Evidence writing style](evidence-writing-style.md)
- [Debug failing checks](debug-failing-checks.md)
- [How suites work](how-suites-work.md)
