# CI report consumption

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: describe how CI consumes control-plane outputs and enforces gates.

## Consumption model

- CI jobs run lane-appropriate control-plane commands.
- Jobs parse JSON reports for pass/fail and artifact references.
- Gate outcome is derived from report contract, not log scraping.

## Concrete lane examples

- `make ci-fast` runs `check run --suite ci_fast --format json` for short local-or-CI feedback.
- `make ci-pr` runs `check run --suite ci_pr --format json` for pull-request enforcement.
- `make ci-nightly` runs `check run --suite ci_nightly --include-internal --include-slow --format json` for broader scheduled coverage.
- `make docs-build` and `make docs-reference-check` produce docs-specific evidence that CI can archive and diff.

## CI guarantees

- Required checks fail closed.
- Missing required report fields fail the lane.
- Artifact paths are stable for review and triage.

## What CI should not do

- It should not infer success from colored console text.
- It should not bypass control-plane wrappers with ad-hoc scripts.
- It should not rewrite or mutate evidence artifacts after generation.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- ci --help
```

## Next steps

- [Reports contract](reports-contract.md)
- [Debug failing checks](debug-failing-checks.md)
- [How suites work](how-suites-work.md)
