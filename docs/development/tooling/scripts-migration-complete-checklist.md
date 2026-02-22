# Control-Plane Migration Complete Checklist

- Owner: `platform`
- Stability: `active`

## Exit Criteria

- [x] `atlasctl` command surface exists and is tested.
- [x] Lockfile-pinned Python dependencies are enforced.
- [x] `make scripts-check` and `make scripts-test` are mandatory gates.
- [x] Script output schemas are validated in CI.
- [ ] Legacy script-tree paths are fully removed.
- [ ] Makefiles contain zero transitional legacy-script path exceptions.

## Current Status

Migration is in strict transition mode. The final removal gate remains pending until legacy shell/python shims are fully eliminated.
