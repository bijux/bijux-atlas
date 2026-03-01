# CONTRACT (bijux-atlas-core)

- Owner: bijux-atlas-core
- Stability: stable

## Inputs

- Public API calls and documented configuration values.
- Declared file inputs from crate fixtures and documented interfaces.

## Outputs

- Deterministic results for identical inputs.
- Public outputs documented in:
- [Crate docs index](docs/index.md)
- [Central docs index](../../docs/index.md)

## Invariants

- Behavior is deterministic and reproducible.
- Contract changes must remain explicit and reviewable.
- Relative documentation links must resolve.

## Effects policy

- No implicit network access.
- Filesystem writes are explicit and bounded.
- Subprocess execution is explicit and justified.

## Error policy

- Errors are stable at the contract layer.
- Error messages include actionable remediation where feasible.

## Versioning/stability

- Public behavior changes require explicit versioning rationale.
- Backward-incompatible changes must be documented before release.

## Tests expectations

- Unit tests cover core behavior and invariants.
- Contract checks run in CI and must remain green.

## Dependencies allowed

- Dependencies must be justified by crate scope and interface boundaries.
- Cross-crate coupling must follow architecture direction rules.

## Anti-patterns

- Hidden side effects.
- Undocumented interface changes.
- Non-deterministic output generation.

## Bench expectations

- Benchmarks must be deterministic and runnable from a clean checkout.
- Performance expectations are tracked in crate docs where benches exist.

## Public API surface

- Public API is defined by exported interfaces and documented contract behavior.
- Breaking public API changes require explicit compatibility and release notes.
