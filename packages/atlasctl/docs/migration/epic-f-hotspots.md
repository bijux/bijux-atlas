# Epic F Hotspot Inventory (Top 10)

Date: 2026-02-21
Source: `atlasctl policies culprits files-per-dir` + `atlasctl policies culprits modules-per-dir`

## Densest directories

1. `packages/atlasctl/src/atlasctl/core`
- files/dir: 12 (budget 10, FAIL)
- modules/dir: 11 (budget 9, FAIL)

2. `packages/atlasctl/src/atlasctl/checks/layout/artifacts`
- files/dir: 10 (budget 12, WARN)

3. `packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks`
- files/dir: 10 (budget 12, WARN)

4. `packages/atlasctl/src/atlasctl/checks/layout/ops/checks`
- files/dir: 10 (budget 12, WARN)

5. `packages/atlasctl/src/atlasctl/checks/repo/enforcement`
- files/dir: 10 (budget 10, WARN)
- modules/dir: 9 (budget 10, WARN)

6. `packages/atlasctl/src/atlasctl/contracts`
- files/dir: 10 (budget 10, WARN)
- modules/dir: 9 (budget 10, WARN)

7. `packages/atlasctl/src/atlasctl/observability/contracts/metrics`
- files/dir: 9 (budget 10, WARN)

8. `packages/atlasctl/src/atlasctl/reporting`
- files/dir: 10 (budget 10, WARN)
- modules/dir: 9 (budget 10, WARN)

9. `packages/atlasctl/tests/cli`
- files/dir: 10 (budget 10, WARN)
- modules/dir: 10 (budget 10, WARN)

10. `packages/atlasctl/tests/commands`
- files/dir: 10 (budget 10, WARN)
- modules/dir: 10 (budget 10, WARN)

## Ownership decision

Observability contract ownership is canonical in:
- `packages/atlasctl/src/atlasctl/observability/contracts/*`

Legacy layout observability contract scripts are removed from:
- `packages/atlasctl/src/atlasctl/checks/layout/policies/observability/*`

## Next splits

- Split `checks/layout/makefiles/checks` by concept (`contracts`, `safety`, `ownership`).
- Split `checks/layout/ops/checks` by concept (`layout`, `surface`, `governance`).
- Keep `checks/layout/policies/*` focused to non-observability layout contracts only.
