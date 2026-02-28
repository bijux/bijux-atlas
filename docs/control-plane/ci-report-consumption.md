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

## CI guarantees

- Required checks fail closed.
- Missing required report fields fail the lane.
- Artifact paths are stable for review and triage.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- ci --help
```

## Next steps

- [Reports contract](reports-contract.md)
- [Debug failing checks](debug-failing-checks.md)
