# Governance Source Of Truth

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the primary governance authority pages so policy does not drift across duplicated explanations.

## Primary Authorities

- `docs/_internal/governance/index.md` is the governance entrypoint.
- `docs/_internal/governance/docs-ownership.md` is the ownership authority.
- `docs/_internal/governance/redirects-contract.md` is the redirect authority.
- `docs/_internal/governance/docs-artifact-contract.md` is the committed-vs-generated artifact authority.
- `docs/_internal/governance/docs-change-classification.md` is the change review authority.

## Rule

When governance text conflicts, the primary authority page wins. Secondary pages should link to these sources
instead of restating policy in competing wording. Navigation, redirect, and committed-artifact changes must defer to
these authority pages rather than creating parallel policy text elsewhere.
