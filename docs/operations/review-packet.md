# Review Packet

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `institutional-reader`
- Stability: `stable`
- Last verified against: `main@93ad533e5a4c4704f3a344db96b083570bb4d4b0`
- Reason to exist: list the exact evidence files to hand to an external reviewer.

## Required files

- `release/evidence/manifest.json`
- `release/evidence/identity.json`
- `release/evidence/bundle.tar`
- `artifacts/ops/<run_id>/reports/ops-simulation-summary.json`
- `artifacts/ops/<run_id>/reports/ops-lifecycle-summary.json`
- `artifacts/ops/<run_id>/reports/ops-drills-summary.json`
- `artifacts/ops/<run_id>/reports/ops-obs-verify.json`

## Recommended attachments

- The matching `ops-drill-<name>.json` files used for the candidate review.
- The latest release candidate checklist and upgrade guide.
