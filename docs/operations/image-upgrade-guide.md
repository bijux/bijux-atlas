# Upgrading Images Safely

1. Validate image manifest and digest registry.
2. Pull by digest, not mutable tag.
3. Run `bijux-dev-atlas runtime self-check --format json` and verify `/healthz`, `/readyz`, `/v1/version`.
4. Roll forward only if readiness and contract checks pass.
5. If any check fails, rollback to previous digest.
