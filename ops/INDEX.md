# Ops Canonical Entrypoints

`atlasctl ops` is the control-plane SSOT for ops workflows. `makefiles/ops.mk` is wrapper-only.

## Canonical CLI

- `./bin/atlasctl ops check`
- `./bin/atlasctl ops gen run`
- `./bin/atlasctl ops gen check`
- `./bin/atlasctl ops pins check`
- `./bin/atlasctl ops pins update`
- `./bin/atlasctl ops env validate`
- `./bin/atlasctl ops env print`
- `./bin/atlasctl ops stack up|down|restart`
- `./bin/atlasctl ops kind up|down|reset|validate|fault <disk-pressure|latency|cpu-throttle>`
- `./bin/atlasctl ops e2e run --suite smoke|k8s-suite|realdata`
- `./bin/atlasctl ops load run --suite mixed-80-20`
- `./bin/atlasctl ops obs verify`
- `./bin/atlasctl ops obs drill --drill <name>`

## Canonical Wrappers

- `atlasctl ops stack up`
- `atlasctl ops stack down`
- `atlasctl ops deploy`
- `atlasctl ops e2e run`
- `atlasctl ops check`
- `atlasctl ops smoke`
- `./bin/atlasctl ops prereqs --report text`
- `./bin/atlasctl ops doctor --report text`
- `atlasctl ops root-lanes`
- `atlasctl ops root-local`
- `atlasctl ops warm`

## Policy

- CI/docs must call `atlasctl ops ...` (or `make ops-*` wrappers), never raw `ops/*/scripts/*`.
- Do not call `ops/*/scripts/*.sh` directly from docs, CI workflows, or root/ci makefiles.
- Legacy `legacy/*` or `*-legacy` entrypoint names are forbidden.

## Contracts

- `ops/datasets/CONTRACT.md`
- `ops/e2e/CONTRACT.md`
- `ops/fixtures/CONTRACT.md`
- `ops/k8s/CONTRACT.md`
- `ops/load/CONTRACT.md`
- `ops/obs/CONTRACT.md`
- `ops/report/CONTRACT.md`
- `ops/run/CONTRACT.md`
- `ops/stack/CONTRACT.md`
