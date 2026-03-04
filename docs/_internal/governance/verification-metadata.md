# Verification Metadata

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the canonical verification marker format for stable docs pages.

## Canonical Format

Use exactly one of the following formats:

- `main@<full_sha>`
- `vX.Y.Z@<full_sha>`

## Rules

- The Git reference must be explicit.
- The commit hash must be the full 40-character SHA.
- Date-only placeholders are not valid verification markers.
- Short SHAs are not valid verification markers.

## Scope

Stable markdown pages must include a `Last verified against` marker in canonical form.
