# Docs Change Classification

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@1c57080c86b4ca4e0e7166fa4423b55dd986d584`
- Reason to exist: define how to classify documentation changes so review scope stays proportional to risk.

## Minor Changes

Treat a docs change as minor when it does not alter the reader spine, redirect behavior, or stable contract language.
Examples: typo fixes, clarified examples, regenerated artifacts with no semantic change, or updated verification hashes.

## Major Changes

Treat a docs change as major when it changes navigation, redirects, published page names, stable policy language, or
operator workflows. These changes need explicit `docs-governance` owner approval before merge.

## Verification

For major changes, rerun docs contracts and `mkdocs build --strict`, then note the affected governance surfaces in
the review summary.
