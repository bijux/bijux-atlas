# Duplicate Topics Checklist

- Owner: `docs-governance`

Use one canonical page per topic and convert other pages to short pointer stubs.

## Canonical Topics

- Architecture boundaries: `docs/architecture/boundaries.md`
- Architecture effects: `docs/architecture/effects.md`
- Boundaries: `docs/architecture/boundaries.md`
- Plugin contract: `docs/contracts/plugin/spec.md`
- Plugin mode: `docs/contracts/plugin/mode.md`
- Immutability and aliases: `docs/product/immutability-and-aliases.md`
- Compatibility: `docs/contracts/compatibility.md`

## Merge Rules

- Do not create parallel docs for existing canonical topics.
- If context differs by component, add a pointer section instead of duplicate prose.
- All docs touched in a change must include an `Owner` line near the top.

## Last Reviewed

Last reviewed is derived from git history (`git-revision-date-localized` plugin).
No manual timestamp fields are allowed in docs pages.
