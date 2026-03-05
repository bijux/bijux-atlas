# Drift Troubleshooting Guide

1. Run `bijux-dev-atlas drift detect --format json`.
2. Group findings by `drift_type` and `severity`.
3. Use `bijux-dev-atlas drift explain <type> --format json` for detector intent.
4. If drift is acceptable short term, add a narrowly scoped ignore rule.
5. Re-run detection and ensure ignored findings are explicit and reviewable.

Exit codes:

- `0`: no findings after ignore rules
- `3`: findings remain
- `1`: command/runtime failure
- `2`: invalid explain selector
