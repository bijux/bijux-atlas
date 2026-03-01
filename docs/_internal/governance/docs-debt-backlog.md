# Docs debt backlog

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: turn generated docs audits into a maintained cleanup queue with explicit status.

> Contributor-only: enforcement machinery.

## Current queue

| Area | Status | Source | Notes |
| --- | --- | --- | --- |
| Broken links | Closed for current stable pages | [Broken links report](../generated/governance-audit/docs-broken-links.csv) | Current report is stale and still references already deleted legacy pages; strict build is clean |
| Dead-end pages | In progress | [Dead-end pages report](../generated/governance-audit/docs-dead-end-pages.txt) | Remaining entries should be reviewed during quarterly cleanup and either linked from a canonical index or removed |
| Delete candidates | Wave 1 completed | [Delete candidates](../generated/governance-audit/docs-top-delete-pages.md) | Root mirrors and quickstart duplicates from the first wave are already removed from the published tree |
| Merge candidates | Wave 1 completed | [Merge candidates](../generated/governance-audit/docs-top-merge-clusters.md) | Canonical pages now own onboarding, contracts, control-plane, and operations narratives |
| Uppercase index cleanup | Wave 1 completed | [Uppercase index cleanup](../generated/governance-audit/docs-uppercase-index-pages.txt) | Published section entrypoints use `index.md`; stale artifact entries must be refreshed with the next audit run |

## Working rule

- New cleanup work starts from generated audit artifacts.
- Once a queue item is complete, update the canonical doc surface and refresh generated inventories before closing it here.
- Do not treat generated audit files as reader-facing truth; they are contributor diagnostics.

## Next steps

- [Docs quarterly cleanup](docs-quarterly-cleanup.md)
- [Docs dashboard](docs-dashboard.md)
