# Redirect Lifecycle And Expiry

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define when redirects are kept or removed.

## Retention

- Keep a redirect while inbound links or published references still reasonably depend on it.
- Remove a redirect only after the replacement path has been stable long enough for consumers to
  update.

## Required Metadata

Every governed redirect must resolve to:

- an owner
- a migration reason
- an expiry date when the redirect is temporary

Those fields are resolved from `docs/_internal/governance/redirect-registry.json`.

## Removal Criteria

- The destination page remains unchanged and valid.
- Internal docs links no longer depend on the old path.
- External references have been updated or the old path is no longer part of supported guidance.

## Verification

Before removing a redirect, run `mkdocs build --strict` and verify no docs entrypoint depends on
the old path. Regenerate `docs/_internal/generated/legacy-url-inventory.md` with
`bijux-dev-atlas docs redirects sync --allow-write` before merging the redirect update.
