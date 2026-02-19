# Ops Tooling Config

Contains operational tooling lockfiles.

- `tool-versions.json`: pinned versions for `kind`, `k6`, `helm`, `kubectl`, `jq`, `yq`.
- `pins/`: split SSOT pin inputs (tools, images, helm, datasets).
- `pins.json`: generated unified reproducibility pins contract.

Validation:

```bash
make ops-tools-check
make pins/check
make doctor
```
