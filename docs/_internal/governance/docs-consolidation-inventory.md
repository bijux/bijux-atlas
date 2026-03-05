# Docs Consolidation Inventory

Inventory classification vocabulary:

- `canonical`: the single preferred page for a concept/domain.
- `supporting`: valid companion page that does not duplicate canonical narrative.
- `redundant`: duplicate page retained only for compatibility redirect behavior.
- `obsolete`: replaced page that should only exist as a redirect source.
- `internal-only`: governance/internal page not intended as public reader flow.

Source of truth: `docs/_internal/governance/docs-consolidation-inventory.json`

## Inventory scope and ownership

- Owner: `docs-governance`
- Update trigger: doc move, merge, split, or major nav change.
- Redirect requirement: every `redundant` or `obsolete` entry must map in `docs/redirects.json`.

## Merge admission rule

A new page can be added only when one of these is true:

1. It extends a canonical page without duplicating concept scope.
2. It declares a concrete consolidation blocker in the pull request rationale.
3. It is intentionally internal-only governance evidence.

If none apply, the content must merge into an existing canonical page.
