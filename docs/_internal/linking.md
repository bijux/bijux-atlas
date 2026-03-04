# Documentation Linking Rules

- Owner: `bijux-atlas-platform`
- Audience: `contributors`

## Allowed Link Directions

- `docs/**` -> `docs/**`
- `docs/reference/ops/**` -> stable `ops/**` specification paths
- `docs/**` -> stable `ops/**` specification paths
- `ops/**` stubs -> canonical `docs/**` pages

## Forbidden Link Directions

- `docs/**` -> `ops/_generated/**`
- `docs/**` -> `ops/_generated.example/**` as SSOT
- `ops/**` -> `docs/_internal/**` for stable user-facing guidance
- `ops/**` -> broad `docs/**` narrative pages except explicit canonical stub targets

## Generated and Example Surfaces

- Markdown under `ops/**/generated/` must be explicitly marked generated.
- `ops/_generated.example/**` is curated example evidence and must be excluded from SSOT authority checks.
