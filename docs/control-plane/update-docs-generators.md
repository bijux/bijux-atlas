# Update docs generators

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define the safe workflow for changing docs generation logic and proving the result is reproducible.

## Workflow

1. Change the generator implementation in `crates/bijux-dev-atlas`.
2. Regenerate outputs with the canonical wrapper.
3. Inspect the diff for unintended page churn.
4. Re-run docs validation and strict build.

## Canonical commands

```bash
make docs-registry
make docs-reference-regenerate
make docs-reference-check
mkdocs build --strict
```

## Review bar

- generator output should change only where the source contract changed
- redirect and canonical links must still resolve
- generated outputs must not leak into reader pages except through approved entrypoints

## Verify success

```bash
make docs-reference-check
```

## Next steps

- [Update schema generators](update-schema-generators.md)
- [Docs change process](../_meta/docs-change-process.md)
- [CLI reference](cli-reference.md)
