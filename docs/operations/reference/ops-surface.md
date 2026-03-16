---
title: Ops Surface Reference
audience: operators
type: reference
status: generated
owner: bijux-atlas-operations
last_reviewed: 2026-03-16
---

# Ops Surface Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Stability: `stable`
- Source-of-truth: `ops/inventory/surfaces.json`, `ops/_generated.example/control-plane-surface-list.json`

## Purpose

Generated ops surface reference derived from inventory surfaces.

## Entry Points

- `ops`
- `ops-artifact-root-check`
- `ops-clean`
- `ops-contracts`
- `ops-contracts-effect`
- `ops-doctor`
- `ops-down`
- `ops-e2e`
- `ops-help`
- `ops-install-plan`
- `ops-k8s`
- `ops-kind-down`
- `ops-kind-up`
- `ops-load`
- `ops-load-plan`
- `ops-load-run`
- `ops-observability`
- `ops-pins-check`
- `ops-pins-update`
- `ops-render`
- `ops-reset`
- `ops-stack`
- `ops-status`
- `ops-tools-verify`
- `ops-trace-debug`
- `ops-traces-check`
- `ops-up`
- `ops-validate`

## bijux-dev-atlas Commands

- `bijux-dev-atlas contract run --mode effect --domain ops --allow-subprocess --allow-network`
- `bijux-dev-atlas contract run --mode static --domain ops`
- `bijux-dev-atlas ops --help`
- `bijux-dev-atlas ops actions list`
- `bijux-dev-atlas ops actions run`
- `bijux-dev-atlas ops cache prune`
- `bijux-dev-atlas ops cache status`
- `bijux-dev-atlas ops clean`
- `bijux-dev-atlas ops datasets fetch`
- `bijux-dev-atlas ops datasets lint-ids`
- `bijux-dev-atlas ops datasets lock`
- `bijux-dev-atlas ops datasets pin`
- `bijux-dev-atlas ops datasets qc diff`
- `bijux-dev-atlas ops datasets qc summary`
- `bijux-dev-atlas ops datasets validate`
- `bijux-dev-atlas ops datasets verify`
- `bijux-dev-atlas ops deploy apply`
- `bijux-dev-atlas ops deploy plan`
- `bijux-dev-atlas ops deploy rollback`
- `bijux-dev-atlas ops directory-budgets-check`
- `bijux-dev-atlas ops doctor`
- `bijux-dev-atlas ops down`
- `bijux-dev-atlas ops e2e run`
- `bijux-dev-atlas ops e2e validate`
- `bijux-dev-atlas ops e2e validate-results`
- `bijux-dev-atlas ops env print`
- `bijux-dev-atlas ops env validate`
- `bijux-dev-atlas ops explain`
- `bijux-dev-atlas ops gen check`
- `bijux-dev-atlas ops gen run`
- `bijux-dev-atlas ops helm-env`
- `bijux-dev-atlas ops install --kind --apply`
- `bijux-dev-atlas ops install --kind --apply --plan`
- `bijux-dev-atlas ops install --kind --plan`
- `bijux-dev-atlas ops k8s apply-config`
- `bijux-dev-atlas ops k8s check`
- `bijux-dev-atlas ops k8s contracts`
- `bijux-dev-atlas ops k8s diff`
- `bijux-dev-atlas ops k8s render`
- `bijux-dev-atlas ops k8s test`
- `bijux-dev-atlas ops k8s validate`
- `bijux-dev-atlas ops k8s validate-configmap-keys`
- `bijux-dev-atlas ops k8s-checks-layout`
- `bijux-dev-atlas ops k8s-flakes-check`
- `bijux-dev-atlas ops k8s-surface-generate`
- `bijux-dev-atlas ops k8s-test-contract`
- `bijux-dev-atlas ops k8s-test-lib-contract`
- `bijux-dev-atlas ops kind down`
- `bijux-dev-atlas ops kind fault`
- `bijux-dev-atlas ops kind reset`
- `bijux-dev-atlas ops kind up`
- `bijux-dev-atlas ops kind validate`
- `bijux-dev-atlas ops layer-drift-check`
- `bijux-dev-atlas ops lint`
- `bijux-dev-atlas ops list`
- `bijux-dev-atlas ops load check`
- `bijux-dev-atlas ops load compare`
- `bijux-dev-atlas ops load plan`
- `bijux-dev-atlas ops load plan mixed`
- `bijux-dev-atlas ops load run`
- `bijux-dev-atlas ops naming-check`
- `bijux-dev-atlas ops no-direct-script-usage-check`
- `bijux-dev-atlas ops observe check`
- `bijux-dev-atlas ops observe drill`
- `bijux-dev-atlas ops observe lint`
- `bijux-dev-atlas ops observe report`
- `bijux-dev-atlas ops observe up`
- `bijux-dev-atlas ops observe validate`
- `bijux-dev-atlas ops observe verify`
- `bijux-dev-atlas ops observe verify --verbose`
- `bijux-dev-atlas ops pins check`
- `bijux-dev-atlas ops pins update`
- `bijux-dev-atlas ops pins update --i-know-what-im-doing`
- `bijux-dev-atlas ops policy-audit`
- `bijux-dev-atlas ops prereqs`
- `bijux-dev-atlas ops profiles validate`
- `bijux-dev-atlas ops render --target kind --check`
- `bijux-dev-atlas ops reset`
- `bijux-dev-atlas ops restart`
- `bijux-dev-atlas ops root-lanes`
- `bijux-dev-atlas ops root-local`
- `bijux-dev-atlas ops schema-check`
- `bijux-dev-atlas ops smoke`
- `bijux-dev-atlas ops stack check`
- `bijux-dev-atlas ops stack down`
- `bijux-dev-atlas ops stack report`
- `bijux-dev-atlas ops stack restart`
- `bijux-dev-atlas ops stack status`
- `bijux-dev-atlas ops stack status --target pods`
- `bijux-dev-atlas ops stack up`
- `bijux-dev-atlas ops stack validate`
- `bijux-dev-atlas ops stack versions-sync`
- `bijux-dev-atlas ops status --target pods`
- `bijux-dev-atlas ops suites-check`
- `bijux-dev-atlas ops surface`
- `bijux-dev-atlas ops tool-versions-check`
- `bijux-dev-atlas ops up`
- `bijux-dev-atlas ops validate`
- `bijux-dev-atlas ops verify-tools`
- `bijux-dev-atlas ops warm`
- `bijux-dev-atlas ops warm-dx`

## Actions

| Action ID | Domain | Command | Dry Run | Artifacts |
| --- | --- | --- | --- | --- |
| `ops.actions.list` | `actions` | `bijux-dev-atlas ops actions list` | `optional` | `artifacts_root_only` |
| `ops.actions.run` | `actions` | `bijux-dev-atlas ops actions run` | `required` | `artifacts_root_only` |
| `ops.cache.prune` | `cache` | `bijux-dev-atlas ops cache prune` | `required` | `artifacts_root_only` |
| `ops.cache.status` | `cache` | `bijux-dev-atlas ops cache status` | `optional` | `artifacts_root_only` |
| `ops.datasets.fetch` | `datasets` | `bijux-dev-atlas ops datasets fetch` | `not_applicable` | `none` |
| `ops.datasets.lint-ids` | `datasets` | `bijux-dev-atlas ops datasets lint-ids` | `not_applicable` | `none` |
| `ops.datasets.lock` | `datasets` | `bijux-dev-atlas ops datasets lock` | `not_applicable` | `none` |
| `ops.datasets.pin` | `datasets` | `bijux-dev-atlas ops datasets pin` | `not_applicable` | `none` |
| `ops.datasets.qc.diff` | `datasets` | `bijux-dev-atlas ops datasets qc diff` | `optional` | `artifacts_root_only` |
| `ops.datasets.qc.summary` | `datasets` | `bijux-dev-atlas ops datasets qc summary` | `not_applicable` | `none` |
| `ops.datasets.validate` | `datasets` | `bijux-dev-atlas ops datasets validate` | `optional` | `artifacts_root_only` |
| `ops.datasets.verify` | `datasets` | `bijux-dev-atlas ops datasets verify` | `not_applicable` | `none` |
| `ops.deploy.apply` | `deploy` | `bijux-dev-atlas ops deploy apply` | `required` | `artifacts_root_only` |
| `ops.deploy.plan` | `deploy` | `bijux-dev-atlas ops deploy plan` | `not_applicable` | `none` |
| `ops.deploy.rollback` | `deploy` | `bijux-dev-atlas ops deploy rollback` | `required` | `artifacts_root_only` |
| `ops.e2e.run` | `e2e` | `bijux-dev-atlas ops e2e run` | `required` | `artifacts_root_only` |
| `ops.e2e.validate` | `e2e` | `bijux-dev-atlas ops e2e validate` | `optional` | `artifacts_root_only` |
| `ops.e2e.validate-results` | `e2e` | `bijux-dev-atlas ops e2e validate-results` | `not_applicable` | `none` |
| `ops.env.print` | `env` | `bijux-dev-atlas ops env print` | `not_applicable` | `none` |
| `ops.env.validate` | `env` | `bijux-dev-atlas ops env validate` | `optional` | `artifacts_root_only` |
| `ops.gen.check` | `gen` | `bijux-dev-atlas ops gen check` | `optional` | `artifacts_root_only` |
| `ops.gen.run` | `gen` | `bijux-dev-atlas ops gen run` | `required` | `artifacts_root_only` |
| `ops.helm-env` | `helm-env` | `bijux-dev-atlas ops helm-env` | `not_applicable` | `none` |
| `ops.k8s.apply-config` | `k8s` | `bijux-dev-atlas ops k8s apply-config` | `not_applicable` | `none` |
| `ops.k8s.check` | `k8s` | `bijux-dev-atlas ops k8s check` | `optional` | `artifacts_root_only` |
| `ops.k8s.contracts` | `k8s` | `bijux-dev-atlas ops k8s contracts` | `not_applicable` | `none` |
| `ops.k8s.diff` | `k8s` | `bijux-dev-atlas ops k8s diff` | `optional` | `artifacts_root_only` |
| `ops.k8s.render` | `k8s` | `bijux-dev-atlas ops k8s render` | `not_applicable` | `none` |
| `ops.k8s.validate` | `k8s` | `bijux-dev-atlas ops k8s validate` | `optional` | `artifacts_root_only` |
| `ops.k8s.validate-configmap-keys` | `k8s` | `bijux-dev-atlas ops k8s validate-configmap-keys` | `not_applicable` | `none` |
| `ops.kind.down` | `kind` | `bijux-dev-atlas ops kind down` | `required` | `artifacts_root_only` |
| `ops.kind.fault` | `kind` | `bijux-dev-atlas ops kind fault` | `required` | `artifacts_root_only` |
| `ops.kind.reset` | `kind` | `bijux-dev-atlas ops kind reset` | `required` | `artifacts_root_only` |
| `ops.kind.up` | `kind` | `bijux-dev-atlas ops kind up` | `required` | `artifacts_root_only` |
| `ops.kind.validate` | `kind` | `bijux-dev-atlas ops kind validate` | `optional` | `artifacts_root_only` |
| `ops.load.check` | `load` | `bijux-dev-atlas ops load check` | `optional` | `artifacts_root_only` |
| `ops.load.compare` | `load` | `bijux-dev-atlas ops load compare` | `not_applicable` | `none` |
| `ops.load.run` | `load` | `bijux-dev-atlas ops load run` | `required` | `artifacts_root_only` |
| `ops.observe.check` | `observe` | `bijux-dev-atlas ops observe check` | `optional` | `artifacts_root_only` |
| `ops.observe.drill` | `observe` | `bijux-dev-atlas ops observe drill` | `not_applicable` | `none` |
| `ops.observe.lint` | `observe` | `bijux-dev-atlas ops observe lint` | `not_applicable` | `none` |
| `ops.observe.report` | `observe` | `bijux-dev-atlas ops observe report` | `optional` | `artifacts_root_only` |
| `ops.observe.up` | `observe` | `bijux-dev-atlas ops observe up` | `required` | `artifacts_root_only` |
| `ops.observe.validate` | `observe` | `bijux-dev-atlas ops observe validate` | `optional` | `artifacts_root_only` |
| `ops.observe.verify` | `observe` | `bijux-dev-atlas ops observe verify` | `not_applicable` | `none` |
| `ops.pins.check` | `pins` | `bijux-dev-atlas ops pins check` | `optional` | `artifacts_root_only` |
| `ops.pins.update` | `pins` | `bijux-dev-atlas ops pins update` | `not_applicable` | `none` |
| `ops.profiles.validate` | `profiles` | `bijux-dev-atlas ops profiles validate` | `optional` | `artifacts_root_only` |
| `ops.root.clean` | `root` | `bijux-dev-atlas ops clean` | `not_applicable` | `none` |
| `ops.root.directory-budgets-check` | `root` | `bijux-dev-atlas ops directory-budgets-check` | `optional` | `artifacts_root_only` |
| `ops.root.doctor` | `root` | `bijux-dev-atlas ops doctor` | `not_applicable` | `none` |
| `ops.root.down` | `root` | `bijux-dev-atlas ops down` | `required` | `artifacts_root_only` |
| `ops.root.explain` | `root` | `bijux-dev-atlas ops explain` | `optional` | `artifacts_root_only` |
| `ops.root.help` | `root` | `bijux-dev-atlas ops --help` | `optional` | `artifacts_root_only` |
| `ops.root.k8s-checks-layout` | `root` | `bijux-dev-atlas ops k8s-checks-layout` | `not_applicable` | `none` |
| `ops.root.k8s-flakes-check` | `root` | `bijux-dev-atlas ops k8s-flakes-check` | `optional` | `artifacts_root_only` |
| `ops.root.k8s-surface-generate` | `root` | `bijux-dev-atlas ops k8s-surface-generate` | `not_applicable` | `none` |
| `ops.root.k8s-test-contract` | `root` | `bijux-dev-atlas ops k8s-test-contract` | `not_applicable` | `none` |
| `ops.root.k8s-test-lib-contract` | `root` | `bijux-dev-atlas ops k8s-test-lib-contract` | `not_applicable` | `none` |
| `ops.root.layer-drift-check` | `root` | `bijux-dev-atlas ops layer-drift-check` | `optional` | `artifacts_root_only` |
| `ops.root.lint` | `root` | `bijux-dev-atlas ops lint` | `not_applicable` | `none` |
| `ops.root.list` | `root` | `bijux-dev-atlas ops list` | `optional` | `artifacts_root_only` |
| `ops.root.naming-check` | `root` | `bijux-dev-atlas ops naming-check` | `optional` | `artifacts_root_only` |
| `ops.root.no-direct-script-usage-check` | `root` | `bijux-dev-atlas ops no-direct-script-usage-check` | `optional` | `artifacts_root_only` |
| `ops.root.policy-audit` | `root` | `bijux-dev-atlas ops policy-audit` | `not_applicable` | `none` |
| `ops.root.prereqs` | `root` | `bijux-dev-atlas ops prereqs` | `not_applicable` | `none` |
| `ops.root.restart` | `root` | `bijux-dev-atlas ops restart` | `required` | `artifacts_root_only` |
| `ops.root.root-lanes` | `root` | `bijux-dev-atlas ops root-lanes` | `not_applicable` | `none` |
| `ops.root.root-local` | `root` | `bijux-dev-atlas ops root-local` | `not_applicable` | `none` |
| `ops.root.schema-check` | `root` | `bijux-dev-atlas ops schema-check` | `optional` | `artifacts_root_only` |
| `ops.root.smoke` | `root` | `bijux-dev-atlas ops smoke` | `not_applicable` | `none` |
| `ops.root.suites-check` | `root` | `bijux-dev-atlas ops suites-check` | `optional` | `artifacts_root_only` |
| `ops.root.surface` | `root` | `bijux-dev-atlas ops surface` | `not_applicable` | `none` |
| `ops.root.tool-versions-check` | `root` | `bijux-dev-atlas ops tool-versions-check` | `optional` | `artifacts_root_only` |
| `ops.root.up` | `root` | `bijux-dev-atlas ops up` | `required` | `artifacts_root_only` |
| `ops.root.validate` | `root` | `bijux-dev-atlas ops validate` | `not_applicable` | `artifacts_root_only` |
| `ops.root.warm` | `root` | `bijux-dev-atlas ops warm` | `not_applicable` | `none` |
| `ops.root.warm-dx` | `root` | `bijux-dev-atlas ops warm-dx` | `not_applicable` | `none` |
| `ops.stack.check` | `stack` | `bijux-dev-atlas ops stack check` | `optional` | `artifacts_root_only` |
| `ops.stack.down` | `stack` | `bijux-dev-atlas ops stack down` | `required` | `artifacts_root_only` |
| `ops.stack.report` | `stack` | `bijux-dev-atlas ops stack report` | `optional` | `artifacts_root_only` |
| `ops.stack.restart` | `stack` | `bijux-dev-atlas ops stack restart` | `required` | `artifacts_root_only` |
| `ops.stack.status` | `stack` | `bijux-dev-atlas ops stack status` | `optional` | `artifacts_root_only` |
| `ops.stack.up` | `stack` | `bijux-dev-atlas ops stack up` | `required` | `artifacts_root_only` |
| `ops.stack.validate` | `stack` | `bijux-dev-atlas ops stack validate` | `optional` | `artifacts_root_only` |
| `ops.stack.versions-sync` | `stack` | `bijux-dev-atlas ops stack versions-sync` | `not_applicable` | `none` |

## See Also

- `ops/_generated.example/control-plane-surface-list.json` (generated surface evidence)
- `ops/inventory/surfaces.json` (machine truth)

## Regenerate

- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`

## Stability

This page is generated from the ops surface inventory. Change the inventory first, then regenerate the docs snapshot.
