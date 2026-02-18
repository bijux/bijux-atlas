# Load Baselines

- Owner: `bijux-atlas-operations`

Contains named baseline artifacts for perf regression comparison.

- `local.json`: baseline captured from local compose/kind environment.
- `ci-runner.json`: baseline captured from nightly CI runner.

Baseline updates require explicit approval in PR and should include rationale.

Baseline metadata should include:
- tool versions (`k6`, `kind`, `kubectl`, `helm`)
- machine profile (`os`, `arch`, `cpu`, `memory_gb`)
