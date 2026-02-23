# ops/run Deletion Plan (Post-Command Parity)

Purpose: retire `ops/run/**` behavior scripts after atlasctl CLI parity is achieved,
while allowing temporary migration shims with explicit milestones.

## Milestones

1. Product parity (complete)
   - `product` CLI lanes exist and `makefiles/product.mk` delegates to atlasctl.
   - `product.mk` has no direct `ops/run` invocations.

2. Area parity (in progress)
   - `atlasctl ops k8s|load|obs|stack <actions>` cover operational behavior currently implemented in `ops/run/**`.
   - Each area exposes `atlasctl ops <area> check`.
   - New behavior lands in the atlasctl CLI only.

3. Migration guardrails (active)
   - `ops/run` count non-increasing.
   - `ops/run` non-allowlisted scripts fail checks.
   - `make` recipes may not call the CLI execution path on `ops/run/...`.

4. Deletion phase
   - For each script family: command parity test + output parity where needed + docs updated.
   - Delete script and remove from transitional allowlist.
   - Regenerate command/check registries and update migration inventory docs.

5. Final state
   - `ops/run/**` contains only approved fixtures/docs (or is removed entirely).
   - Temporary `atlasctl ops run-script` shim removed.

## Temporary Approval Rules

- Any temporary migration script kept under `ops/run/**` must have:
  - owner
  - justification
  - expiry date / milestone
  - replacement atlasctl CLI target
- Expired temporary scripts must fail migration checks and be removed or renewed intentionally.

## Burn-down Reporting

- Use `atlasctl internal migration status`
- Use `atlasctl internal migration burn-down`
- Artifacts are written under `artifacts/reports/atlasctl/` and evidence roots
