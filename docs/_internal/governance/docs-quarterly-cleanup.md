# Docs quarterly cleanup

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define the recurring cleanup routine for stable docs quality.

> Contributor-only: enforcement machinery.

## Cadence

- Run once per quarter.
- Run immediately after large docs migrations or section restructures.

## Required checks

1. Review [Docs debt backlog](docs-debt-backlog.md).
2. Verify strict site build passes: `mkdocs build --strict`.
3. Review generated audit artifacts linked from [Docs dashboard](docs-dashboard.md).
4. Confirm stable pages still carry owner and freshness metadata.
5. Remove or relink dead-end pages.
6. Remove obsolete redirects and add new redirects for moved pages.

## Evidence

- Commit updated docs, redirects, and generated artifacts together.
- Record the cleanup outcome in the pull request summary or release evidence notes.

## Service levels

- Urgent factual corrections: within `24h`.
- Normal corrections: within `72h`.
- Stable page freshness review: within `180` days.
- Quarterly prune budget review: each quarter must produce a docs shrink report and duplicate-topic report with explicit follow-up owners.

## Next steps

- [Docs operating model](docs-operating-model.md)
- [Docs debt backlog](docs-debt-backlog.md)
