---
title: Install Matrix
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Install Matrix

The install matrix records which values files, profiles, and suites must hold
for supported installation paths.

## Purpose

Use the install matrix to decide which profile is supported for a target
environment, which suite must pass, and whether install, upgrade, or rollback
is part of the governed path.

## Source of Truth

- `ops/k8s/install-matrix.json`
- `ops/schema/k8s/install-matrix.schema.json`
- `ops/k8s/values/profiles.json`

## Supported Profiles

`ops/k8s/install-matrix.json` currently defines these supported profile entries:

| Profile | Values file | Target environment | Required suite | Offline or air-gapped status | Promotion status |
| --- | --- | --- | --- | --- | --- |
| `ci` | `ops/k8s/values/ci.yaml` | fast CI validation | `install-gate` | no | supported for gate checks |
| `dev` | `ops/k8s/values/dev.yaml` | local or Kind development | `install-gate` | no | supported for local validation |
| `ingress` | `ops/k8s/values/ingress.yaml` | ingress-specific validation | `nightly` | no | supported when ingress paths are under review |
| `kind` | `ops/k8s/values/kind.yaml` | deterministic Kind cluster | `k8s-suite` | no | supported for realistic cluster validation |
| `local` | `ops/k8s/values/local.yaml` | local single-operator installs | `install-gate` | no | supported for workstation validation |
| `multi-registry` | `ops/k8s/values/multi-registry.yaml` | registry-heavy validation | `nightly` | no | supported for broader integration review |
| `offline` | `ops/k8s/values/offline.yaml` | offline or air-gapped install path | `k8s-suite` | yes | supported for disconnected installs |
| `perf` | `ops/k8s/values/perf.yaml` | load and scaling environments | `nightly` | no | supported for performance promotion review |
| `prod` | `ops/k8s/values/prod.yaml` | production rollout | `nightly` | no | supported for production promotion review |
| `profile-baseline` | `ops/k8s/values/profile-baseline.yaml` | shared chart baseline validation | `k8s-suite` | no | supported as the common baseline |

## Governed Scenarios

The same matrix also defines the named scenarios operators are expected to run:

- install: `install-profile-baseline`, `install-ci`, `install-kind`,
  `install-offline`, and `install-perf`
- upgrade: `upgrade-kind`, `upgrade-offline`, and `upgrade-perf`
- rollback: `rollback-kind`, `rollback-offline`, and `rollback-perf`

Upgrade and rollback entries require both `baseline_ref` and `target_ref`,
which makes the promotion comparison explicit instead of implied.

## How to Validate

1. Confirm the matrix entry matches the target environment and intended use.
2. Validate the file against `ops/schema/k8s/install-matrix.schema.json`.
3. Run the named suite for the selected profile.
4. If the path is `upgrade` or `rollback`, confirm both baseline and target
   references are recorded.
5. Carry the resulting evidence into rollout and release review.

## Failure Modes

- an install path exists in practice but is absent from the matrix
- a profile is reused in a different environment without a declared suite
- an offline claim is made without using the `offline` profile path
- upgrade or rollback is attempted without an explicit baseline comparison

## Related Contracts and Assets

- `ops/k8s/install-matrix.json`
- `ops/schema/k8s/install-matrix.schema.json`
- `ops/k8s/values/profiles.json`
