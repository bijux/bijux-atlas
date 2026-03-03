---
title: Evidence Retention Policy
audience: operator
type: policy
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Evidence Retention Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@5fcfe93aaeed218cb75ecb5c143ee3129fbe4bcf`
- Last changed: `2026-03-03`
- Reason to exist: define how long release evidence must be retained and what can be pruned.

Related ops contracts: `OPS-ROOT-023`, `REL-EVID-005`.

## Retention Rules

- Keep release-candidate evidence until the candidate is either promoted or rejected.
- Keep promoted release evidence for the full supported lifetime of that release.
- Keep at least the manifest, identity, chart package, bundle tarball, and SBOM set for every promoted release.
- Temporary local work files may be pruned after the final bundle is verified.

## Storage Rules

- Store the final bundle in a write-once location when possible.
- Store checksums alongside the bundle when mirroring it outside the repository workspace.
- Do not retain unredacted debug logs in the evidence package.
