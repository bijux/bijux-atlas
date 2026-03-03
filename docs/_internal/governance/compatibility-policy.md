# Compatibility Policy

- Owner: `bijux-atlas-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define the governed compatibility boundary for runtime configuration, chart values, profiles, report schemas, and documentation URLs.

## Breaking changes

- Env keys: removing a key, renaming a key without an overlap window, or changing requiredness.
- Chart values: removing a key, changing type, removing a compatibility alias before its removal target, or changing a safety default.
- Profile keys: removing a consumed key or dropping an alias before the registry says it can be removed.
- Report schemas: removing required fields, changing field type, or changing report identity without a migration plan.
- Docs URLs: moving a reader URL without a redirect or reusing a stable URL for a different meaning.

## Rename rules

- Every governed rename must appear in `configs/governance/deprecations.yaml`.
- Env-key renames must keep the old and new names in the env allowlist during the overlap window and update the relevant docs.
- Chart-value renames must keep both old and new keys accepted by `ops/k8s/charts/bijux-atlas/values.schema.json` during the overlap window.
- Docs URL moves must have a redirect entry in `docs/redirects.json`.

## Validation surfaces

- `governance deprecations validate`
- `artifacts/governance/deprecations-summary.json`
- `artifacts/governance/compat-warnings.json`
