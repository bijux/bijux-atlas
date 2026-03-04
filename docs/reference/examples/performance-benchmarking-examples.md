# Performance Benchmarking Examples

- Owner: `reference`
- Stability: `stable`

## Example Commands

```bash
ops/load/tools/compare-load-report.sh \
  ops/load/baselines/system-load-baseline.json \
  ops/load/baselines/system-load-baseline.json

ops/load/tools/detect-performance-regression.sh \
  ops/load/baselines/system-load-baseline.json \
  ops/load/baselines/system-load-baseline.json

ops/load/tools/generate-regression-report.sh \
  ops/load/baselines/system-load-baseline.json \
  ops/load/baselines/system-load-baseline.json \
  /tmp/performance-regression-report.json
```
