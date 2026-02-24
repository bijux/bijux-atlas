# bijux-dev-atlas

Rust control-plane tool for Atlas development checks and workflows.

## Goals
- Replace atlasctl check/gate surfaces incrementally.
- Keep one runner and one registry contract for checks.
- Keep effect boundaries explicit through adapters.
- Own `ops` control-plane behavior through `bijux dev atlas ...` routing.

## Policy Domains
- `root`
- `workflows`
- `make`
- `ops`
- `docs`
- `configs`
- `docker`
- `crates`

## Non-goals
- No direct dependency on `packages/atlasctl` runtime.
- No shell-script check execution as SSOT.

## Plugin dispatch
- Binary: `bijux-dev-atlas`
- Umbrella route: `bijux dev atlas <args>` should execute `bijux-dev-atlas <args>`.

## Ops ownership contract
- `ops/` is SSOT data: manifests, schemas, docs, and inventories.
- Runtime behavior for ops workflows belongs to `bijux-dev-atlas`.
- `makefiles/ops.mk` must delegate to `bijux dev atlas ...`, not atlasctl surfaces.

## Output contract
- `list`, `explain`, `doctor`, and `run` support `--format text|json`.
- `run` additionally supports `--format jsonl`.
- `--out <path>` is supported on all commands.
- Default output is deterministic and excludes timestamps.
- `--print-policies` prints resolved dev governance policies as stable JSON.

## Root discovery and write policy
- `--repo-root` is optional; when omitted the binary walks upward from cwd until it finds `Cargo.toml` or `.git`.
- `--artifacts-root` defaults to `<repo-root>/artifacts`.
- Write-capable checks are constrained to `artifacts/atlas-dev/<run-id>/...`.

## Effect capabilities
- Deny-by-default for `subprocess`, `git`, `network`, and `fs_write`.
- Flags `--allow-subprocess`, `--allow-git`, `--allow-network`, and `--allow-write` opt in.
