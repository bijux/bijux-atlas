# Coverage Configs

Coverage thresholds and rationale for quality gates.

## Files

- `thresholds.toml`
  - Consumer: coverage policy gates and release readiness checks.

## Threshold Policy

- Thresholds are intentionally conservative to keep signal stable across environments.
- Changes require rationale in the same commit.

## Why

Predictable thresholds avoid noisy regressions while preserving minimum confidence.

## Verification

```bash
make coverage
```
