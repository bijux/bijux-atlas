# Profiles Matrix

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide a compact comparison of the governed profile surfaces.

| Profile | Intended Use | Risk | Network Policy | HPA | Digest Required | Storage |
| --- | --- | --- | --- | --- | --- | --- |
| `ci` | `ci` | `low` | `disabled` | `disabled` | `no` | `ephemeral` |
| `kind` | `kind` | `medium` | `cluster-aware` | `disabled` | `no` | `ephemeral` |
| `offline` | `offline` | `medium` | `disabled` | `disabled` | `no` | `ephemeral` |
| `perf` | `perf` | `high` | `cluster-aware` | `enabled` | `yes` | `ephemeral` |
| `prod` | `prod` | `high` | `cluster-aware` | `enabled` | `no` | `ephemeral` |
| `prod-minimal` | `prod` | `high` | `cluster-aware` | `enabled` | `yes` | `ephemeral` |
| `prod-ha` | `prod` | `high` | `cluster-aware` | `enabled` | `yes` | `ephemeral` |
| `prod-airgap` | `prod` | `high` | `disabled` | `disabled` | `yes` | `ephemeral` |
