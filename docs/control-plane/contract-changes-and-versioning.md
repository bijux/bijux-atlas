# Contract changes and versioning

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define the compatibility rules for changing control-plane contracts and report schemas.

## Rules

- additive fields are preferred over renames or removals
- required key changes need an explicit version bump and consumer review
- make wrappers and CI readers must be updated in the same change when contract meaning changes
- breaking changes require a migration note in the destination reader page

## Typical change classes

- additive: new optional report field
- compatible tightening: clearer validation with unchanged consumer contract
- breaking: renamed command, removed key, or changed required semantics

## Verify success

```bash
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
make ci-pr
```

## Next steps

- [Reports contract](reports-contract.md)
- [Extensibility and stability levels](extensibility-and-stability-levels.md)
- [Update schema generators](update-schema-generators.md)
