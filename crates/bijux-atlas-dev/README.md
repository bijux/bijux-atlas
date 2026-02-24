# bijux-atlas-dev

Rust control-plane tool for Atlas development checks and workflows.

## Goals
- Replace atlasctl check/gate surfaces incrementally.
- Keep one runner and one registry contract for checks.
- Keep effect boundaries explicit through adapters.

## Non-goals
- No direct dependency on `packages/atlasctl` runtime.
- No shell-script check execution as SSOT.

## Plugin dispatch
- Binary: `bijux-atlas-dev`
- Umbrella route: `bijux dev atlas <args>` should execute `bijux-atlas-dev <args>`.
