# Evidence writing style

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: ensure failures are actionable, reproducible, and lane-aware.

## Style rules

- State failure in one concrete sentence.
- Include direct remediation hint.
- Include reproducible command.
- Include artifact path when output exists.

## Verify success

Generated failure messages should let another contributor reproduce and validate a fix in one iteration.

## Next steps

- [Reports contract](reports-contract.md)
- [Debug failing checks](debug-failing-checks.md)
