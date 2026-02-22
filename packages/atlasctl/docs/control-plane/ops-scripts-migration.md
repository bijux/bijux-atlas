# ops/run Migration Inventory (Behavior -> atlasctl)

Purpose: migrate behavior out of `ops/run/**` shell scripts into `atlasctl` command implementations so `ops/` remains data/manifests/contracts oriented.

Status legend:
- `migrated`: behavior implemented in atlasctl command(s)
- `planned`: mapping chosen, migration not yet complete
- `legacy`: candidate for deprecation/removal after command parity

Classification legend:
- `glue`: thin wrapper that composes existing commands/tools
- `logic`: script contains command flow/business logic that should move into atlasctl
- `legacy`: deprecated or superseded lane wrapper

## Inventory

| Script | Class | Status | Future atlasctl command / home |
|---|---|---:|---|
| `ops/run/artifacts-open.sh` | glue | deleted (atlasctl-native) | `atlasctl ops artifacts open` |
| `ops/run/cache-prune.sh` | glue | deleted (atlasctl-native) | `atlasctl ops cache prune` |
| `ops/run/cache-status.sh` | glue | deleted (atlasctl-native) | `atlasctl ops cache status` |
| `ops/run/ci-fast.sh` | glue | deleted (atlasctl-native) | `atlasctl ci fast` |
| `ops/run/ci-nightly.sh` | glue | deleted (atlasctl-native) | `atlasctl ci nightly` |
| `ops/run/clean.sh` | glue | deleted (replaced by `cleanup` + `ops cache prune`) | `atlasctl cleanup` + `atlasctl ops cache prune` |
| `ops/run/configmap-drift-report.sh` | logic | planned | `atlasctl ops k8s configmap-drift-report` |
| `ops/run/contract-check.sh` | glue | planned | `atlasctl contracts check` |
| `ops/run/contract-report.py` | logic | planned | `atlasctl contracts report` |
| `ops/run/datasets-verify.sh` | glue | deleted (atlasctl-native) | `atlasctl ops datasets verify` |
| `ops/run/deploy-atlas.sh` | logic | planned | `atlasctl ops deploy` |
| `ops/run/doctor.sh` | glue | planned | `atlasctl doctor` / `atlasctl make doctor` |
| `ops/run/down.sh` | glue | deleted (atlasctl-native guard + stack-down) | `atlasctl ops down` |
| `ops/run/e2e-smoke.sh` | glue | deleted (folded into `ops/run/e2e.sh`) | `atlasctl ops e2e run --scenario smoke` |
| `ops/run/e2e.sh` | glue | planned | `atlasctl ops e2e run` |
| `ops/run/evidence-bundle.sh` | logic | deleted (atlasctl-native) | `atlasctl reporting bundle` |
| `ops/run/evidence-open.sh` | glue | deleted (unused wrapper) | `make evidence/open` / `atlasctl reporting artifact-index` |
| `ops/run/k8s-apply-config.sh` | logic | planned | `atlasctl ops k8s apply-config` |
| `ops/run/k8s-restart.sh` | glue | planned | `atlasctl ops restart` / `atlasctl ops stack restart` |
| `ops/run/k8s-suite.sh` | glue | deleted (dead wrapper) | `atlasctl suite run ops-deploy` / `atlasctl ops k8s check` |
| `ops/run/k8s-tests.sh` | glue | deleted (dead wrapper) | `atlasctl ops k8s check` |
| `ops/run/k8s-validate-configmap-keys.sh` | logic | planned | `atlasctl ops k8s validate-configmap-keys` |
| `ops/run/load-smoke.sh` | glue | deleted (folded into orchestrator) | `atlasctl ops load smoke` / `atlasctl load smoke` |
| `ops/run/load-suite.sh` | glue | planned | `atlasctl ops load run` |
| `ops/run/obs-up.sh` | logic | deleted (atlasctl-native) | `atlasctl ops obs up` |
| `ops/run/obs-validate.sh` | glue | deleted (atlasctl-native) | `atlasctl ops obs validate` |
| `ops/run/obs-verify.sh` | glue | planned | `atlasctl obs verify` / `atlasctl ops obs verify` |
| `ops/run/ops-check.sh` | glue | planned | `atlasctl ops check` |
| `ops/run/ops-smoke.sh` | glue | planned | `atlasctl ops smoke` |
| `ops/run/prereqs.sh` | glue | planned | `atlasctl make prereqs` |
| `ops/run/report.sh` | glue | planned | `atlasctl report ...` |
| `ops/run/root-lanes.sh` | legacy | legacy | split across `atlasctl dev/ci/product/ops` |
| `ops/run/root-local.sh` | legacy | legacy | split across `atlasctl dev/ops` |
| `ops/run/stack-down.sh` | glue | planned | `atlasctl ops stack down` |
| `ops/run/stack-up.sh` | glue | planned | `atlasctl ops stack up` |
| `ops/run/undeploy.sh` | glue | planned | `atlasctl ops undeploy` |
| `ops/run/warm-dx.sh` | glue | planned | `atlasctl ops datasets fetch` / `atlasctl ops warm dx` |
| `ops/run/warm-entrypoint.sh` | glue | planned | `atlasctl ops datasets fetch` |
| `ops/run/warm.sh` | glue | deleted (folded into warm-entrypoint) | `atlasctl ops datasets fetch` |
| `ops/run/root/root_artifacts_open.sh` | legacy | deleted (inlined in `make artifacts-open`) | `make artifacts-open` / future `atlasctl artifacts open` |
| `ops/run/root/root_quick.sh` | legacy | deleted (inlined in `make quick`) | `atlasctl dev fmt && lint && test` |

## Product Script Mapping (migrated and deleted)

| Script | Class | Status | atlasctl command |
|---|---|---:|---|
| `ops/run/product/product_bootstrap.sh` | logic | deleted (migrated) | `atlasctl product bootstrap` |
| `ops/run/product/product_docker_build.sh` | logic | deleted (migrated) | `atlasctl product docker build` |
| `ops/run/product/product_docker_push.sh` | logic | deleted (migrated) | `atlasctl product docker push` |
| `ops/run/product/product_docker_release.sh` | logic | deleted (migrated) | `atlasctl product docker release` |
| `ops/run/product/product_docker_check.sh` | logic | deleted (migrated) | `atlasctl product docker check` |
| `ops/run/product/product_chart_package.sh` | glue | deleted (migrated) | `atlasctl product chart package` |
| `ops/run/product/product_chart_verify.sh` | glue | deleted (migrated) | `atlasctl product chart verify` |
| `ops/run/product/product_chart_validate.sh` | logic | deleted (migrated) | `atlasctl product chart validate` |
| `ops/run/product/product_rename_lint.sh` | glue | deleted (migrated) | `atlasctl product naming lint` |
| `ops/run/product/product_docs_lint_names.sh` | logic | deleted (migrated) | `atlasctl product docs naming-lint` |

## Migration Rules

- New behavior goes in `atlasctl` command/effects modules, not `ops/run/*.sh`.
- `makefiles/*.mk` wrappers should call `./bin/atlasctl ...` only.
- Command implementations should emit stable JSON reports under evidence roots using `run_id`.
- Shell scripts in `ops/run/` should be reduced to deprecated stubs or removed after parity and docs updates.
