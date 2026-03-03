# Compatibility Review Checklist

- Owner: `bijux-atlas-governance`
- Type: `checklist`
- Audience: `reviewers`
- Stability: `stable`

## Checklist

- Confirm the change is recorded in `configs/governance/deprecations.yaml` when a stable name moves.
- Confirm production profile compatibility stays clean or is covered by an active exception.
- Confirm report schema changes have migration notes.
- Confirm docs URL changes have redirects.
- Confirm breaking changes have release notes and satisfy chart version policy.
