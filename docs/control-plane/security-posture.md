# Security posture

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: explain how the control-plane limits privilege while still enabling build, validation, and release workflows.

## Security model

- capabilities are explicit, not inferred
- effectful commands fail closed without approval flags
- CI consumes structured reports instead of parsing shell output
- external tools run through stable wrappers instead of free-form scripts

## Risk boundaries

- filesystem writes are limited to generated outputs and evidence roots
- network access is reserved for checks that actually need it
- subprocess execution is gated so review can see when a doc or release workflow depends on external tools

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- contracts ops --mode static --format json
```

## Next steps

- [Capabilities model](capabilities-model.md)
- [Adding external tooling](adding-external-tooling.md)
- [Control-plane architecture](control-plane-architecture.md)
