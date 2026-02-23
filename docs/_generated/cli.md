# Atlasctl CLI Commands (generated)

Generated from the unified registry spine.

## check

- `check` (platform): run registered check suites and individual checks [artifacts/evidence/, check]
## configs

- `configs` (platform): configs checks and inventories [configs, configs/]
## contracts

- `contracts` (platform): contracts domain commands [configs/contracts/, contracts]
## deps

- `deps` (platform): dependency management commands [deps, packages/atlasctl/requirements.lock.txt, python3]
## dev

- `dev` (platform): dev control-plane group command surface [artifacts/evidence/, dev]
## docs

- `docs` (docs): docs checks and generation [docs, docs/, docs/_generated/, mkdocs]
## doctor

- `doctor` (platform): show tooling and context diagnostics [artifacts/evidence/, doctor, python3]
## fix

- `fix` (platform): apply repository hygiene and consistency fixes [artifacts/, fix, python3]
## gate

- `gate` (ops): run gate contracts and lane checks [artifacts/evidence/, configs/gates/, gate, make]
## gates

- `gates` (ops): run gate contracts and lane checks [artifacts/evidence/, configs/gates/, gates, make]
## gen

- `gen` (platform): artifact generation commands [docs/_generated/, gen, packages/atlasctl/tests/goldens/, python3]
## install

- `install` (platform): installation checks and doctor commands [artifacts/evidence/, git, install, make, python3]
## inventory

- `inventory` (repo): generate and verify inventories [docs/_generated/, inventory]
## k8s

- `k8s` (ops): k8s checks and suites [helm, k8s, kubectl, ops/k8s/]
## layout

- `layout` (repo): layout domain commands [docs/development/repo-layout.md, layout, makefiles/]
## lint

- `lint` (platform): lint suite runner [lint, ops/_lint/]
## list

- `list` (platform): list checks/commands/suites from canonical registries [artifacts/evidence/, list]
## load

- `load` (ops): load and perf suites [k6, load, ops/load/]
## make

- `make` (platform): make target wrappers and make contract tooling [configs/make/, make, makefiles/]
## obs

- `obs` (ops): observability checks and drills [obs, ops/obs/]
## ops

- `ops` (ops): ops checks and suite orchestration [artifacts/evidence/, helm, k6, kubectl, ops]
## owners

- `owners` (platform): owner id registry and validation commands [configs/meta/owners.json, owners]
## packages

- `packages` (platform): package domain commands [packages, packages/]
## policies

- `policies` (platform): policy relaxations and bypass checks [configs/policy/, policies]
## registry

- `registry` (platform): registry domain commands [configs/ops/pins/, registry]
## release

- `release` (platform): release readiness checklist commands [artifacts/evidence/, python3, release]
## repo

- `repo` (repo): repository stats and density reports [artifacts/reports/atlasctl/, configs/policy/, repo]
## report

- `report` (platform): unified report and scorecard commands [artifacts/evidence/, report]
## run

- `run-id` (platform): generate and print run identifiers [artifacts/evidence/, run]
## stack

- `stack` (ops): stack lifecycle and checks [kubectl, ops/stack/, stack]
## suite

- `suite` (platform): atlasctl-native suite runner [artifacts/reports/atlasctl/, python3, suite]
## test

- `test` (platform): run canonical atlasctl test suites [artifacts/isolate/, python3, test]
