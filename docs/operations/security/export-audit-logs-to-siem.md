# Export Audit Logs To Institutional SIEM

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operators`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: describe the supported paths for forwarding governed audit records to external
  monitoring systems.

## Prereqs

- Audit logging is enabled.
- `audit.sink` is set to `otel` or `file`.
- The receiving SIEM is approved for operator-only access.

## Verify

- Confirm `artifacts/security/audit-verify.json` reports `status: ok`.
- Confirm the forwarding pipeline preserves JSON structure and field names from the governed inventory.
- Confirm secret-pattern scans remain clean after export configuration changes.

## Supported Paths

- Preferred: `audit.sink=otel` and forward the `atlas_audit` JSON payloads through the existing OTEL path.
- Secondary: `audit.sink=file` with `rbac.auditLogReader.enabled=true` and a dedicated reader service
  account for collection.

## Rollback

- Disable the export pipeline before turning off audit logging.
- Remove temporary audit-reader RBAC bindings when file export is no longer needed.
