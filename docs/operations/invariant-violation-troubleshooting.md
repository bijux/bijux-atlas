# Invariant Violation Troubleshooting

Run:

```bash
bijux-dev-atlas invariants run --format json
```

Troubleshooting workflow:

1. Identify failing IDs and severity from `results`.
2. Use `bijux-dev-atlas invariants explain <id> --format json`.
3. Resolve filesystem or metadata mismatches at the `violations[].path` location.
4. Re-run `invariants run` until status is `ok`.

Operational note:

- Exit code `3` indicates invariant failures.
- Exit code `0` indicates all invariants and registry completeness checks passed.
