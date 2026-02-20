# Scripts Compatibility Policy

Patch releases must remain backward compatible for:

- documented CLI command names,
- documented flags,
- JSON output schemas with required core fields.

## 2-Step Removal Policy
- Step 1 (`deprecate`): add shim entry to `configs/layout/script-shim-expiries.json` with owner, issue, replacement, migration doc, and expiry.
- Step 2 (`remove`): delete shim and remove exceptions/allowlists once expiry date is reached.

## Shim Rules
- Shims must print a deprecation warning and migration doc link.
- Shims must `exec` the replacement command and pass through exit codes.
- Shims must not write artifacts or mutate repo files.
- Shims must be POSIX `sh` compatible and deterministic.

## Not Allowed To Shim
- Security-sensitive commands that modify trust, keys, signatures, or policy bypass behavior.
- Commands that mutate production infra state outside audited runbooks.
- Commands whose replacement changes authentication/authorization semantics.

Breaking interface changes require a major version bump and migration guidance.
