# Load Reproducibility

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines variance controls for reproducible load measurements.

## Why

Prevents noisy comparisons and false regression alarms.

## Contracts

- Scenario SSOT: `ops/load/scenarios/*.json`
- Query SSOT + lock: `ops/load/queries/pinned-v1.json`, `ops/load/queries/pinned-v1.lock`
- Suite manifest: `ops/load/suites/suites.json`
- Result metadata must include git SHA, image digest, dataset hash, dataset release.
- Tool versions are pinned in `configs/ops/tool-versions.json` and validated by `make ops-tools-check`.

## Sources of variance

- Host CPU throttling and thermal state.
- Background process interference.
- Network stack differences between compose and kind.
- Dataset/cache warmness.

## Variance Controls

- Run `make ops-tools-check` before load runs (pinned tools).
- Use pinned dataset/query lock (`ops/load/queries/pinned-v1.lock`).
- Keep cluster profile stable between baseline and candidate.
- Compare against approved baseline only (`ops/load/baselines/*.json`).
- Re-run `make ops-load-smoke` twice before concluding regression.

## How to verify

```bash
$ make ops-load-smoke
$ make ops-load-full
$ make ops-load-smoke
```

Expected output: deterministic suite selection and contract-valid result artifacts.

## See also

- [Load Suites](suites.md)
- [Load CI Policy](ci-policy.md)
- `ops-load-full`

- Reference scenario: `mixed.json`
