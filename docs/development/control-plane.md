# Control-plane for contributors

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d09a3c7f`
- Reason to exist: explain how the control-plane keeps repository truth enforceable in local and CI workflows.

## What this page covers

- How static and effect lanes enforce correctness without hidden scripts.
- How capability flags gate filesystem, subprocess, and network effects.
- How reports become CI gates and review evidence.

## How it enforces truth

1. Contracts and checks execute through `bijux-dev-atlas` command surfaces.
2. Required suites fail closed and block merges on contract violations.
3. Outputs are deterministic and archived under `artifacts/run/<run_id>/`.
4. Control-plane policy avoids side-channel execution outside declared capabilities.

## Static vs effect in contributor flow

- Static mode: tree shape, metadata, schema, links, and deterministic formatting checks.
- Effect mode: tool-driven checks such as docs build, docker build, and helm/kube validation.
- Required effect checks must declare capabilities and required tools explicitly.

## Capabilities model in practice

- `--allow-subprocess` for external command execution.
- `--allow-network` for network-dependent validations.
- `--allow-write` for generated artifacts under approved output roots.
- Missing required capabilities is a hard error, not a silent skip.

## Reports and CI integration

- `check run` emits machine-readable status and violation details.
- CI consumes report fields, not console formatting.
- Evidence links point reviewers to exact artifact files and line-level violations.

## Where to go next

- [Control-plane index](../control-plane/index.md)
- [Static and effect mode](../control-plane/static-and-effect-mode.md)
- [Capabilities model](../control-plane/capabilities-model.md)
- [Reports contract](../control-plane/reports-contract.md)
