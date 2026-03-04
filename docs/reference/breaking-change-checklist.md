---
title: Breaking change checklist
audience: contributors
type: guide
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - release
related:
  - docs/reference/crate-release-policy.md
  - docs/operations/release/index.md
---

# Breaking change checklist

1. Confirm the change is actually breaking at API/contract level.
2. Add migration guidance to docs and changelog.
3. Update `CHANGELOG.md` `Breaking Changes` section.
4. Ensure release tag and version satisfy semver policy.
5. Regenerate and validate release notes.
6. Regenerate and validate release manifest and evidence.
7. Run `bijux dev atlas release validate`.
8. Verify compatibility and deprecation policy links are updated.

## Version bump rule

- Breaking public API change requires a semver-major bump.
- Non-breaking additive changes use minor/patch according to impact.
