# Why effects are gated

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: explain why filesystem, network, and subprocess effects are explicitly controlled.

## Why gating exists

- Prevent hidden side effects in runtime-critical paths.
- Keep CI and release behavior reproducible across lanes.
- Reduce blast radius during incident response and rollback.

## Examples

- Subprocess calls only through control-plane surfaces with explicit capability flags.
- Network access disabled by default for checks that do not require external calls.
- Filesystem writes confined to known artifact/output directories.

## Terminology used here

- Lane: [Glossary](../glossary.md)
- Control-plane: [Glossary](../glossary.md)

## Next steps

- [Effects](effects.md)
- [Boundaries](boundaries.md)
- [Development control plane](../development/control-plane.md)
