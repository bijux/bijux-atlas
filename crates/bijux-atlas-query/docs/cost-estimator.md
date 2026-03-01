# Cost Estimator Calibration

Current model combines:
- query class base cost (`cheap`, `medium`, `heavy`)
- limit contribution
- region span contribution
- prefix selectivity heuristic

Calibration policy:
- Keep `max_work_units` conservative enough to reject pathological requests.
- Recalibrate when real-data p95/p99 changes materially.
- Any formula change must include updated tests and benchmark notes.

Safety invariant:
- Cost checks run before SQL execution and before expensive serialization.
