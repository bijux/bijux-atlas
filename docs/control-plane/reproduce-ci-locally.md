# How to reproduce CI locally

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: give one deterministic path from a failing CI lane back to a local control-plane command.

## Lane-to-command map

- docs lane: `make docs-build`
- PR lane: `make ci-pr`
- nightly lane: `make ci-nightly`
- docs registry drift: `make docs-registry`
- ops contract lane: `cargo run -q -p bijux-dev-atlas -- contracts ops --mode static --format json`

## Reproduction workflow

1. Start with the narrowest wrapper that matches the CI lane.
2. Reproduce in JSON mode when the lane consumes reports.
3. If the wrapper fails, rerun the underlying `bijux-dev-atlas` command only when you need narrower selection.

## Verify success

```bash
make ci-pr
make docs-build
```

## Next steps

- [Lane matrix](lane-matrix.md)
- [Common failure messages](common-failure-messages.md)
- [CI report consumption](ci-report-consumption.md)
