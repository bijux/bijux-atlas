# Rename Env Keys Safely

- Owner: `bijux-atlas-governance`
- Type: `how-to`
- Audience: `contributor`
- Stability: `stable`

## Example

- Add the old and new keys to `configs/contracts/env.schema.json`.
- Add a deprecation entry in `configs/governance/deprecations.yaml` with `surface: env-key`.
- Update the relevant docs to mention the new key.
- Keep the old key valid until `removal_target`.
- Run `governance deprecations validate` and `governance breaking validate`.
