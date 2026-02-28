# Debug failing checks

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: provide a reproducible triage workflow for failing control-plane checks.

## Triage flow

1. Reproduce with the exact lane command.
2. Inspect structured report output first.
3. Apply smallest fix that restores contract behavior.
4. Re-run focused check and then lane baseline.

## Practical commands

- `make ci-pr` when a pull-request lane fails
- `cargo run -q -p bijux-dev-atlas -- check run --tag lint --format json` for focused tag reproduction
- `make docs-build` when the failure came from docs build or preview
- `make docs-registry` when the failure came from registry or metadata drift

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- docs inventory --allow-write --allow-subprocess --format json
```

## Next steps

- [Evidence writing style](evidence-writing-style.md)
- [Known limitations](known-limitations.md)
- [Reports contract](reports-contract.md)
