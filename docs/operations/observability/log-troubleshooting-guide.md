# Log Troubleshooting Guide

1. Run `bijux-dev-atlas observe logs explain --format json`.
2. Confirm required schema fields are present.
3. Check `sample_validation` for `redaction_violation`, `invalid_level`, and `unknown_log_class`.
4. If redaction violations appear, enable or fix `ATLAS_LOG_REDACTION_ENABLED` flow.
5. If class mapping violations appear, register event prefixes in log classification contract.
