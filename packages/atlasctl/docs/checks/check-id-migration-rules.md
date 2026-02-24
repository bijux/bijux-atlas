# Check ID Migration Rules

Canonical check IDs use this format:

- `checks_<domain>_<area>_<name>`

Migration policy:

1. Add/keep canonical ID in python registry composition (`checks/domains/*.py` + `checks/registry.py`).
2. Keep legacy dotted IDs temporarily via `legacy_id` metadata.
3. Record non-derived renames in `configs/policy/check-id-migration.json`.
4. Regenerate registry artifacts with `./bin/atlasctl gen checks-registry`.
5. Update goldens and docs that reference renamed checks.

CLI compatibility:

- Canonical: `./bin/atlasctl check run --id checks_repo_cli_argparse_policy`
- Legacy (temporary): `./bin/atlasctl check run --id repo.argparse_policy --legacy-id`

Use `./bin/atlasctl check rename-report --json` to view active alias mappings and expiry.
