# Plugin-Mode Entrypoint (Kubernetes)

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Atlas containers run in plugin mode.

Canonical runtime command:

```bash
/app/bijux-atlas atlas serve
```

Helm alignment:

- `values.yaml` sets `server.command` to `[/app/bijux-atlas, atlas, serve]`.
- Deployment template uses `.Values.server.command` as container command.
- Init prewarm path also uses plugin command style.

This keeps runtime semantics consistent with umbrella/plugin contract.
## Referenced chart values keys

- `values.server`

## See also

- `ops-ci`
