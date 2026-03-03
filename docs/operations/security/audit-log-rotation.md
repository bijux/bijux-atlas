# Audit Log Rotation And Export

Atlas rotates file-sink audit logs before the active file exceeds the configured byte budget.

## Rotation

- Active path: `ATLAS_AUDIT_FILE_PATH`
- Rotated path: `<active>.1`
- Budget: `ATLAS_AUDIT_MAX_BYTES`

Only one rotated copy is kept in the current governed runtime.

## Export

- For local review, copy the active file and the rotated file together.
- For centralized collection, prefer `audit.sink=otel` and forward the structured payloads to the
  institutional collector.
