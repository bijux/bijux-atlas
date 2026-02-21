# Shell Policy

`atlasctl` enforces a quarantine policy for shell checks:

- No `.sh` files are allowed under `packages/atlasctl/src/atlasctl/**`.
- Transitional shell probes live only under `ops/vendor/layout-checks/`.
- Python is the preferred implementation for new checks.
- Core logic may not invoke `subprocess.run(['bash', ...])` or `subprocess.run(['sh', ...])`.
- Any rare shell probe exception must be documented in `configs/policy/shell-probes-allowlist.txt`.
