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

## Typical ownership

- `helm`, `kubectl`, `kind`, and `kubeconform` show up in deploy, conformance, and docs-preview-adjacent validation flows.
- `syft` and `trivy` belong to security or supply-chain style checks, not default local loops.
- Wrappers such as `make docs-build` and `make ci-nightly` are the preferred public entrypoints when those tools are required.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- validate --help
make ci-nightly
```

## Next steps

- [Capabilities model](capabilities-model.md)
- [Known limitations](known-limitations.md)
- [Static and effect mode](static-and-effect-mode.md)
