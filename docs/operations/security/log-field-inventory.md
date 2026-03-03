# Audit Log Field Inventory

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operators`
- Stability: `stable`
- Last verified against: `main@f5f3bd4471d8bf4dbf13fef4381ef6bfda2480a2`
- Reason to exist: explain the governed audit log field inventory and where it is generated.

## Source Of Truth

- Safe fields registry: `configs/observability/log-safe-fields.yaml`
- Sensitive fields registry: `configs/security/data-classification.yaml`
- Generated runtime report: `artifacts/security/log-field-inventory.json`

## Verify

- Run `cargo run -q -p bijux-dev-atlas -- security validate --format json`.
- Confirm `artifacts/security/log-field-inventory.json` exists and `OBS-LOG-INV-001=true`.

## Current Safe Fields

- `event_id`
- `event_name`
- `timestamp_policy`
- `timestamp_unix_s`
- `sink`
- `principal`
- `action`
- `resource_kind`
- `resource_id`
- `status`
- `decision`
- `reason`
- `route`
- `source`
- `outcome`
- `auth_mode`
- `admin_endpoints_enabled`
- `audit_enabled`
- `catalog_configured`
