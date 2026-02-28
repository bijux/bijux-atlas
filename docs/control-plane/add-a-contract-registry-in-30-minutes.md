# Add a contract registry in 30 minutes

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: provide a practical workflow for adding a new contract registry surface.

## Workflow

1. Define canonical registry file path and schema contract.
2. Add registry reader and validation hooks in control-plane commands.
3. Add tests for missing/invalid registry behavior.
4. Add docs references and ownership metadata.

## Minimal recipe

1. Pick one durable file path and schema owner before writing code.
2. Wire the registry into `docs registry` or the relevant domain command instead of adding a side parser.
3. Fail closed on missing or invalid registry entries.
4. Update the canonical reader page that explains the registry surface.

## Verify success

```bash
make docs-registry
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
```

## Next steps

- [Reports contract](reports-contract.md)
- [Add a gate policy](add-a-gate-policy.md)
- [Docs change process](../_meta/docs-change-process.md)
