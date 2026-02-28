# How to add a new crate

- Owner: `architecture`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: provide a checklist and verification flow for introducing a new crate.

## Checklist

1. Define the crate purpose in one sentence and assign owner.
2. Choose layer placement and allowed dependencies.
3. Add contract boundaries for new behaviors.
4. Add docs entry in [Crates map](crates-map.md).
5. Add tests for boundary and behavior expectations.

## Verify success

```bash
make check
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
```

A successful addition keeps dependency rules intact and passes docs/contracts checks.

## Next steps

- [Workspace boundaries rules](workspace-boundaries-rules.md)
- [What belongs where](what-belongs-where.md)
- [Development contributing](../development/contributing.md)
