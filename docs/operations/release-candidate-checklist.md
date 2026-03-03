---
title: Release Candidate Checklist
audience: operator
type: checklist
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Release Candidate Checklist

- Owner: `bijux-atlas-operations`
- Type: `checklist`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@5fcfe93aaeed218cb75ecb5c143ee3129fbe4bcf`
- Last changed: `2026-03-03`
- Reason to exist: provide the minimum release gate for evidence-backed release candidates.

## Checklist

1. Run `ops evidence collect` and confirm it exits successfully.
2. Run `ops evidence verify release/evidence/bundle.tar` and confirm it exits successfully.
3. Confirm `release/evidence/manifest.json` lists the expected chart package, SBOMs, and report paths.
4. Confirm the release candidate has the expected lifecycle and simulation summaries attached when those workflows were executed.
5. Confirm no redacted log file still contains a common secret marker.

## Verify Command

```bash
cargo run -q -p bijux-dev-atlas -- ops evidence verify release/evidence/bundle.tar --allow-write --format json
```

## Review Focus

- Institutions usually expect traceability, reproducibility, rollback evidence, and SBOM coverage.
- Any missing optional scan report should be called out explicitly in the release notes rather than hidden.
