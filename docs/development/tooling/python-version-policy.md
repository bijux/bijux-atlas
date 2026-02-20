# Python Version Policy

- Owner: `platform`
- Stability: `active`

## Policy

- Python tooling version is pinned in `configs/ops/pins/tools.json` under `tools.python3`.
- `bijux-atlas-scripts` requires Python `>=3.10` and is validated through `tools/bijux-atlas-scripts/pyproject.toml`.
- Lockfiles are mandatory for deterministic installs:
  - `tools/bijux-atlas-scripts/requirements.lock.txt`
  - `packages/bijux-atlas-scripts/requirements.lock.txt` (if package path is used)
- Rust policy remains in `rust-toolchain.toml`; use `make tooling-versions` to print both Rust and Python versions.

## Verification

```bash
make tooling-versions
make scripts-lock-check
```
