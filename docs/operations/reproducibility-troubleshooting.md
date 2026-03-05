# Reproducibility Troubleshooting

Run:

```bash
bijux-dev-atlas reproduce run --format json
bijux-dev-atlas reproduce verify --format json
bijux-dev-atlas reproduce status --format json
```

If `verify` fails:

1. Inspect `missing_required_scenarios`.
2. Check `failure_classification` entries.
3. Rebuild evidence at `artifacts/reproducibility/run-report.json`.
