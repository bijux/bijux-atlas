# Documentation Navigation Policy

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define one canonical navigation model and enforce bounded complexity.

## Rules

- Reader entrypoints are `docs/index.md` and `docs/start-here.md`.
- `docs/start-here.md` is the only onboarding funnel.
- `docs/_internal/governance/index.md` is the top-level governance authority in nav.
- Section entrypoints use `index.md` only.
- Navigation depth must not exceed 8.
- Page length warning starts above 500 lines.
- Readability warning starts above average sentence length of 28 words.

## Enforcement

Run `bijux-dev-atlas docs nav check` after navigation edits.
