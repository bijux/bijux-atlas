# CLI Usage Examples

- Owner: `docs-governance`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: provide stable CLI usage examples for common workflows.

## Examples

### Validate contracts and checks

```bash
make contract-all
make check-all
```

### Run docs integrity checks

```bash
cargo run -q -p bijux-dev-atlas -- docs links --strict --format json
```

### Run test suite

```bash
cargo nextest run
```

## Related

- [Commands](../commands.md)
- [Tutorials](../../tutorials/index.md)
