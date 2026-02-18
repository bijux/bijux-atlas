# Ops Tooling Config

Contains operational tooling lockfiles.

- `tool-versions.json`: pinned versions for `kind`, `k6`, `helm`, `kubectl`, `jq`, `yq`.

Validation:

```bash
make ops-tools-check
make doctor
```
