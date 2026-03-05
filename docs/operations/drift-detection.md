# Drift Detection

Atlas drift detection compares expected repository contracts with observed state and reports classified deviations.

Commands:

```bash
bijux-dev-atlas drift detect --format json
bijux-dev-atlas drift report --format json
bijux-dev-atlas drift coverage --format json
```

Baseline workflow:

```bash
bijux-dev-atlas drift baseline --snapshot-out artifacts/drift/baseline.json --format json
bijux-dev-atlas drift compare --baseline artifacts/drift/baseline.json --format json
```

Ignore rules:

```bash
bijux-dev-atlas drift detect --ignore-file ops/drift/ignore-rules.example.json --format json
```
