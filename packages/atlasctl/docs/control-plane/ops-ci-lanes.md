# Ops CI Lanes

This page maps CI lanes to ops suites and checks.

## Fast PR Lanes

- `suite product --only fast`
- `suite ops --only fast`
- ops lint/runtime checks including suite manifest validation (`ops-load-suite-manifest`, `ops-load-suite-baselines-manifest`)

## Slow / Scheduled Lanes

- `suite run ops --only slow`
- `ops.load.regression`
- `ops.e2e.realdata`
- observability drill-heavy suites

## Policy Notes

- Slow suites must declare why they are slow and a reduction plan in `configs/ops/suites.json`.
- Network-restricted lanes may set `ATLASCTL_OPS_NETWORK_FORBID=1` / `CI_NETWORK_FORBID=1`.

