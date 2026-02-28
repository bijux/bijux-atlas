# Why a control-plane

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: explain why command orchestration and contracts must be centralized.

## Problem it solves

- Replaces hidden script sprawl with explicit commands.
- Makes validation behavior deterministic across local, PR, merge, and release lanes.
- Produces machine-readable evidence for every gate decision.

## What it does not do

- It does not own runtime business logic.
- It does not bypass runtime or API contracts.

## Verify success

Run a lane command and confirm deterministic evidence output.

```bash
cargo run -q -p bijux-dev-atlas -- check --help
```

## Next steps

- [How suites work](how-suites-work.md)
- [Capabilities model](capabilities-model.md)
