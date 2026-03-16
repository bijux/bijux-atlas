# Makes

`makes/` is the governed GNU Make wrapper surface for the repository. It exists to provide stable
entrypoints for common local and CI workflows without letting orchestration logic drift into shell
recipes.

## What This Directory Owns

- Public make targets listed in `makes/root.mk:CURATED_TARGETS`.
- Thin wrappers that dispatch to `bijux dev atlas ...`, `bijux atlas ...`, or approved
  cargo-native lanes in `makes/cargo.mk`.
- Shared make fragments used by the repository root `Makefile`.

## What This Directory Does Not Own

- Business logic, workflow policy, or orchestration branching.
- Hidden side effects behind short shell snippets.
- Extra Markdown pages. `makes/README.md` is the only Markdown document in `makes/`.

## Sources of Truth

- Public target list: `makes/root.mk`
- Public target artifact: `makes/target-list.json`
- Target ownership and workflow mapping: `configs/sources/operations/ops/makes-targets.json`
- Repository automation guide: `docs/06-development/automation-control-plane.md`
- Command reference: `docs/07-reference/automation-command-surface.md`

## Update Rules

- Keep public target names aligned with `makes/root.mk:CURATED_TARGETS`.
- Keep target descriptions in this file aligned with the wrapper recipes.
- Regenerate `makes/target-list.json` with `make makes-target-list` whenever the curated target
  set changes.
- Keep complex control flow in Rust command surfaces, not in make recipes.

## Public Target Reference

- `build`: Build the repository through the governed control plane.
- `ci-fast`: Run the fast CI lane.
- `ci-nightly`: Run the nightly CI lane.
- `ci-pr`: Run the pull-request CI lane.
- `clean`: Remove ephemeral artifacts through the control plane.
- `docker`: Run the Docker verification lane.
- `doctor`: Generate repository diagnostics.
- `help`: Print the curated make surface.
- `k8s-render`: Render Kubernetes manifests through the control plane.
- `k8s-validate`: Validate Kubernetes manifests through the control plane.
- `kind-down`: Delete the deterministic kind cluster.
- `kind-reset`: Recreate the deterministic kind cluster.
- `kind-status`: Report kind cluster readiness.
- `kind-up`: Create or verify the deterministic kind cluster.
- `lint-make`: Run the governed make-required check suite.
- `openapi-generate`: Regenerate the OpenAPI contract.
- `ops-contracts`: Validate static ops contracts.
- `ops-contracts-effect`: Run effectful ops contract validation.
- `registry-doctor`: Validate governed suite registries and mappings.
- `release-plan`: Generate the release readiness plan.
- `release-verify`: Verify a release evidence tarball.
- `root-surface-explain`: Explain the root surface contract.
- `stack-down`: Stop the local ops stack.
- `stack-up`: Start the local ops stack.
- `suites-list`: List the available suite identifiers.
- `tests-all`: Run the deterministic test suite.

## Support Targets

These support targets are intentionally outside the curated public surface but still belong to this
domain.

- `makes-target-list`: Regenerate `makes/target-list.json` for workflows, audits, and governance
  checks.
- Internal `_internal-*` helpers in `makes/entrypoints.mk`: support CI drift detection and focused
  validation without widening the public interface.

## Why This Contract Exists

- Reviewers need one stable place to inspect the public make surface.
- CI and governance checks need one checked-in document for public target coverage.
- Operators need a small wrapper surface that stays aligned with installed Atlas command families.
