# CLI reference

- Owner: `platform`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: provide the generated-at-source command surface reference for `bijux-dev-atlas`.

## Top-level command groups

Generated from `cargo run -q -p bijux-dev-atlas -- --help`.

| Command | Purpose |
| --- | --- |
| `ops` | operator and environment workflows |
| `docs` | docs build, validation, and registry flows |
| `contracts` | contract runners and report emitters |
| `demo` | local demo flows |
| `configs` | config validation and compilation |
| `policies` | policy validation |
| `ci` | CI-oriented entrypoints |
| `check` | check discovery, explanation, and execution |
| `validate` | validation entrypoints |

## Frequently used subcommands

| Command | Source command | Typical use |
| --- | --- | --- |
| `check list` | `cargo run -q -p bijux-dev-atlas -- check list` | inspect available checks, tags, and suites |
| `check run --suite ci_pr` | `cargo run -q -p bijux-dev-atlas -- check run --suite ci_pr --format json` | reproduce PR lane checks |
| `docs registry build` | `cargo run -q -p bijux-dev-atlas -- docs registry build --allow-write --format json` | rebuild docs registry and indexes |
| `docs reference generate` | `cargo run -q -p bijux-dev-atlas -- docs reference generate --allow-subprocess --allow-write --format json` | regenerate reference pages from SSOT inputs |
| `ci validate` | `cargo run -q -p bijux-dev-atlas -- ci validate --format json` | run CI validation surface locally |
| `contracts ops` | `cargo run -q -p bijux-dev-atlas -- contracts ops --mode static --format json` | inspect ops contract status |
| `contracts pr` | `cargo run -q -p bijux-dev-atlas -- contracts pr --format json` | run path-sensitive PR contract selection across domains |
| `contracts doctor` | `cargo run -q -p bijux-dev-atlas -- contracts doctor --format json` | emit coverage-by-domain and constitution drift report |

## Output and flags

- `--format text|json|jsonl` is the standard rendering contract.
- `--allow-subprocess`, `--allow-network`, and `--allow-write` are explicit capability gates.
- `--repo-root`, `--artifacts-root`, and `--run-id` make local and CI runs reproducible.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- --help
cargo run -q -p bijux-dev-atlas -- check --help
cargo run -q -p bijux-dev-atlas -- docs --help
```

## Next steps

- [Lane matrix](lane-matrix.md)
- [How to reproduce CI locally](reproduce-ci-locally.md)
- [Capabilities model](capabilities-model.md)
