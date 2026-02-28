# Reports contract

- Owner: `platform`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: define report schema expectations and versioning behavior.

## Contract guarantees

- Report schema is versioned.
- Required keys are stable for consumers in CI and local tooling.
- Report ordering is deterministic for stable diffs.

## Required report fields

- `schema_version`
- execution summary and status
- lane/context metadata
- artifact paths when applicable

## Verify success

Run a report-producing command and inspect JSON output.

```bash
cargo run -q -p bijux-dev-atlas -- docs inventory --allow-write --allow-subprocess --format json
```

## Next steps

- [CI report consumption](ci-report-consumption.md)
- [Evidence writing style](evidence-writing-style.md)
