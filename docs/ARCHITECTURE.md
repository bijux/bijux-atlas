# Atlasctl Architecture

## Target Layout v1

Canonical package root is `packages/atlasctl/`.

### Package Root Contract

Only these top-level items are allowed in `packages/atlasctl/`:

- `src/`
- `tests/`
- `docs/`
- `pyproject.toml`
- `README.md`
- `LICENSE`

### Source Tree Contract

Canonical source tree is `packages/atlasctl/src/atlasctl/` with:

- `core/`: runtime context, effect boundaries, fs/env/exec primitives.
- `cli/`: argparse wiring, dispatch, output rendering.
- `commands/`: command implementations, each exposing `configure(parser)` and `run(ctx, ns)`.
- `checks/`: registry-based checks (`CheckDef`/`CheckResult`) and domain checks.
- `contracts/`: schema catalog, validation helpers, output contracts.
- `adapters/`: external integration adapters (git/tools/python).
- `legacy/`: read-only migration compatibility surface.

### Command and Check SSOT

- Canonical CLI entrypoints: `python -m atlasctl` and installed `atlasctl` script.
- Canonical command system: `atlasctl.commands.*` modules with `configure()/run()`.
- Canonical check system: `atlasctl.checks.*` registry-driven checks.

### Boundary Choices

- Keep `observability/` as canonical; `obs/` is legacy compatibility only.
- Keep `core/fs.py` as canonical filesystem boundary; no top-level `fs.py` for new code.
- Keep `core/exec.py` as canonical process boundary; no top-level `subprocess.py` for new code.

### Dependency and Lock Policy

- Package manager baseline: `pyproject.toml` first, with lock discipline via pip-compatible lock workflow.
- Lock files are required for CI determinism.

