# Enable Audit Logging And Retention

## Prereqs

- Decide whether audit records should stay on `stdout`, be written to a bounded in-pod file, or be
  exported through OTEL.
- Review `configs/observability/retention.yaml` before enabling persistent file output.

## Configure

- Set `audit.enabled=true`
- Choose `audit.sink=stdout|file|otel`
- When using `file`, keep `audit.persistence.enabled=true` and size the mount for the bounded file
  plus one rotated copy

## Retention

- The default retention contract keeps one rotated audit file and a bounded active file.
- The canonical budget is defined in `configs/observability/retention.yaml`.

## Verify

- Confirm the runtime emits `atlas_audit` JSON records.
- Confirm `artifacts/security/audit-verify.json` reports `status: ok`.

## Rollback

- Set `audit.enabled=false`
- Revert `audit.sink` to `stdout` if file or OTEL output is no longer needed
