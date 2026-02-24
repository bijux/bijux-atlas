# Adapter Boundary

`bijux-dev-atlas-adapters` implements IO ports declared by `bijux-dev-atlas-core`.

## Allowed IO
- Filesystem reads for repo inspection.
- Filesystem writes only under `artifacts/atlas-dev/<run_id>/`.
- Subprocess execution only through the centralized allowlist policy.
- Git execution through explicit adapter calls.

## Forbidden By Default
- Network access is denied by default (`DeniedNetwork`).
- Non-allowlisted subprocess programs are denied.
- Writes outside the artifacts run root are denied.

## Why
- Keep core deterministic and testable.
- Keep effect scope explicit and auditable.
- Prevent accidental host mutation from checks.
