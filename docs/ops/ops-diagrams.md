> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Ops Diagrams

- Owner: bijux-atlas-operations
- Stability: stable

```mermaid
flowchart LR
  inventory[ops inventory] --> validate[ops validate]
  validate --> render[ops render]
  render --> apply[ops k8s apply]
  apply --> conformance[ops k8s conformance]
  conformance --> report[ops report]
```
