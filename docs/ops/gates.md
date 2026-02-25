# Ops Gates

Ops checks use canonical IDs and deterministic enforcement.

- Canonical ID format: `checks_ops_<area>_<name>`
- Ops suites include `checks_ops_*` unless explicitly allowlisted.
- Every check must declare owner, severity, slow flag, and fix hint.
- Governance checks are release-blocking when drift is detected.
