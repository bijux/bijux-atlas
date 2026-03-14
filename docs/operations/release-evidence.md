---
title: Release Evidence
audience: operator
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Release Evidence

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define how release evidence is generated, verified, and reviewed.

Related ops contracts: `OPS-ROOT-023`, `REL-EVID-001`.

## Prereqs

- A clean working tree for the files you intend to package.
- `helm`, `python3`, and the `bijux-dev-atlas` binary available locally.
- `artifacts/docs/site` already built if you want docs output included.

## Install

Generate the evidence bundle from the current working tree:

```bash
cargo run -q -p bijux-dev-atlas -- ops evidence collect --allow-subprocess --allow-write --run-id release_candidate --format json
```

This writes `ops/release/evidence/manifest.json`, `ops/release/evidence/identity.json`, `ops/release/evidence/bundle.tar`, digest-pinned SPDX JSON SBOMs under `ops/release/evidence/sboms/`, and an HTML summary at `ops/release/evidence/index.html`.

## Verify

Validate the generated bundle:

```bash
cargo run -q -p bijux-dev-atlas -- ops evidence verify ops/release/evidence/bundle.tar --allow-write --format json
```

Compare two bundles when you need a structured delta:

```bash
cargo run -q -p bijux-dev-atlas -- ops evidence diff ops/release/evidence/bundle-a.tar ops/release/evidence/bundle-b.tar --allow-write --format json
```

Accepted SBOM formats are `spdx-json` and `cyclonedx-json`. Vulnerability scan reports are optional, but when included they must be attached as reports only and use the governed `json` or `sarif` formats.

## Rollback

If evidence generation fails, discard the incomplete release candidate and regenerate after fixing the failing input:

1. Remove or repair the invalid generated evidence files under `ops/release/evidence/`.
2. Re-run `ops evidence collect`.
3. Re-run `ops evidence verify`.

## What Institutions Expect

- Traceability: `ops/release/evidence/identity.json` ties the bundle to a git SHA and governance version.
- Reproducibility: `ops/release/evidence/bundle.tar` is created with normalized metadata and stable ordering.
- Rollback evidence: lifecycle summaries and readiness history are included when present.
- Supply-chain visibility: digest-pinned images, `ops/docker/bases.lock`, toolchain inventory, and SBOMs are captured together.

## Related Pages

- [Release provenance](release-provenance.md)
- [Release candidate checklist](release-candidate-checklist.md)
- [Evidence retention policy](evidence-retention-policy.md)
- [Log redaction policy](log-redaction-policy.md)
- [Evidence viewer](evidence-viewer.md)
