# Perf Experiments Policy

- Owner: `bijux-atlas-operations`

## What

Defines non-gating performance experiments.

## Why

Avoids experimental drift from blocking production gates.

## Contracts

- Files under `ops/load/evaluations/` are non-gating by default.
- Experiments must not be included in `ops/load/suites/suites.json` `run_in` profiles used by CI/nightly gates unless promoted.
- Promotion requires thresholds and expected metrics in suite SSOT.

## Failure modes

Running experiments as gates can create unstable CI signal.

## Verification

```bash
rg -n "experiments" ops/load/suites/suites.json
```

Expected output shape: no experiment suite in gating profiles.

## See also

- `ops/load/suites/suites.json`
- `docs/operations/load/suites.md`
