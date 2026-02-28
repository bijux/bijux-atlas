# Update schema generators

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define how to update schema-producing control-plane code without breaking downstream references.

## Workflow

1. Change the schema source in code or the canonical SSOT file.
2. Regenerate the dependent docs and indexes.
3. Confirm reference pages still point to the same canonical schema URLs.
4. Run contract and docs validation before commit.

## Canonical commands

```bash
make docs-registry
make docs-reference-regenerate
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
```

## Verify success

- schema index reflects the updated SSOT
- reference pages show the updated schema locations
- contract coverage remains green

## Next steps

- [Update docs generators](update-docs-generators.md)
- [Contract changes and versioning](contract-changes-and-versioning.md)
- [Where truth lives](../development/where-truth-lives.md)
