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

## Verify success

```bash
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
```

## Next steps

- [Reports contract](reports-contract.md)
- [Add a gate policy](add-a-gate-policy.md)
