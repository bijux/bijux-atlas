# Legacy Definition

Legacy in `atlasctl` means pre-1.0 compatibility modules/shims kept only during migration and scheduled for deletion.

## Canonical Legacy Modules And Why They Existed

- `legacy/layout_shell/*`: shell-layout checks before `checks/layout/domains/shell`.
- `legacy/obs/*`: observability checks before `observability/*` became canonical.
- `legacy/report/*`: report assembly code before `reporting/*` consolidation.
- `legacy/effects/*`: early effect-boundary checks before `checks/repo/enforcement/boundaries`.
- `legacy/subprocess.py`: process wrapper before `core/exec.py`.
- `legacy/logging.py`: logging wrapper before `core/logging.py`.
- `legacy/repo_checks_native*`: split repo checks before `checks/repo/*` unification.
- `legacy/ops_runtime*`: ops runtime shards before `commands/ops/*` and `checks/layout/ops/*`.
- `legacy/docs_runtime*`: docs runtime shards before `commands/docs/runtime_chunks/*`.

## Current State

- `packages/atlasctl/src/atlasctl/legacy/` is expected to be absent.
- Any remaining legacy behavior must be represented as internal migration metadata only.
- New code must not import from `atlasctl.legacy`.
