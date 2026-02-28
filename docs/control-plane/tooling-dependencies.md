# Tooling dependencies

- Owner: `platform`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: define external tooling dependencies and gating behavior.

## Common external tools

- `helm`
- `kubectl`
- `kind`
- `syft`
- `trivy`
- `kubeconform`

## Gating rules

- Tool-dependent checks must declare capability needs.
- Missing tools fail with actionable remediation.
- Tooling checks are lane-scoped to avoid unnecessary local burden.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- validate --help
```

## Next steps

- [Capabilities model](capabilities-model.md)
- [Known limitations](known-limitations.md)
