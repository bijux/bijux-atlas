# Migrate Docs URLs Safely

- Owner: `docs-governance`
- Type: `how-to`
- Audience: `contributor`
- Stability: `stable`

## Example

- Add the moved page to `configs/governance/deprecations.yaml` with `surface: docs-url`.
- Add the redirect in `docs/redirects.json`.
- Keep the destination page live and stable.
- Run `governance deprecations validate` and `governance breaking validate`.
