# Docs dashboard

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: provide one contributor entrypoint to generated documentation audits and quality signals.

> Contributor-only: enforcement machinery.

## Generated Audit Surfaces

- [Governance Audit Index](../generated/governance-audit/index.md)
- [Broken links report](../generated/governance-audit/docs-broken-links.csv)
- [Dead-end pages report](../generated/governance-audit/docs-dead-end-pages.txt)
- [Delete candidates](../generated/governance-audit/docs-top-delete-pages.md)
- [Merge candidates](../generated/governance-audit/docs-top-merge-clusters.md)
- [Uppercase index cleanup](../generated/governance-audit/docs-uppercase-index-pages.txt)
- [Docs Quality Dashboard](../generated/docs-quality-dashboard.json)
- [Docs Dependency Graph](../generated/docs-dependency-graph.json)
- [Docs Contract Coverage](../generated/docs-contract-coverage.json)
- [Search Index](../generated/search-index.json)

## Usage Rules

- `_generated/**` is tooling output and never part of reader navigation.
- Do not copy generated artifacts into reader pages.
- Open generated files from this page only for contributor troubleshooting.

## Next steps

- [Governance Readers Guide](readers-guide.md)
- [Docs debt backlog](docs-debt-backlog.md)
- [Generated artifacts](../meta/generated-artifacts.md)
- [Docs Search Tips](docs-search-tips.md)
