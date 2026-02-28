# How suites work

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: define suite and tag selection behavior for predictable check execution.

## Suite model

- A suite groups checks by intent and operational cost.
- Tags allow focused execution without bypassing required checks.
- Lane policy determines which suites are mandatory.

## Selection rules

- Local: fast suites for iterative feedback.
- PR: required suites for policy and contract integrity.
- Merge and release: complete required suites and evidence.

## Verify success

List available check command surfaces.

```bash
cargo run -q -p bijux-dev-atlas -- check --help
```

## Next steps

- [Static and effect mode](static-and-effect-mode.md)
- [CI report consumption](ci-report-consumption.md)
