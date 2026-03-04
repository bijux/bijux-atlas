# Performance Baseline Maintenance

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Purpose

This runbook governs how the committed performance baseline is updated. The baseline is evidence,
not a convenience cache.

## Update Conditions

Only update `ops/report/gene-lookup-baseline.json` after a fresh local deterministic perf run
and an explicit review of the SLO and budget files.

## Procedure

1. Run `bijux-dev-atlas perf validate --format json`.
2. Run `bijux-dev-atlas perf run --scenario gene-lookup --format json`.
3. Compare the new report to the committed baseline with `bijux-dev-atlas perf diff`.
4. Update the committed baseline only when the new result is the accepted reference point and the
   reason is documented in the commit.

## Rollback

If the new baseline is wrong, restore the previous committed baseline and rerun the comparison.
