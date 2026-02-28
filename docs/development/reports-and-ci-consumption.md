# Reports and CI Consumption

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: explain how reports are produced and consumed in CI lanes.

## Flow

1. Control-plane command runs in a lane.
2. Machine-readable report artifacts are written.
3. CI consumers evaluate required gates from report outputs.

## Deterministic Ordering

Report ordering is stable so review diffs reflect behavior changes, not runtime noise.

## JSON Schema Note

Report schema references belong in reference docs; this page links behavior, not full schema dumps.

## Verify Success

Expected report artifacts are present and parseable for required CI lanes.

## What to Read Next

- [CI Overview](ci-overview.md)
- [Control Plane](control-plane.md)
- [Reference](../reference/index.md)
