# Generated Files Registry

Source of truth files:

- `configs/docs/generated-files-registry.json`
- `configs/docs/generated-files-freshness-policy.json`

Purpose:

- registry of generated docs outputs and owning generator command.
- drift protection through `docs verify-generated`.
- freshness protection through maximum generated-file age budget.

Validation path:

```bash
bijux-dev-atlas docs verify-generated --format json
```
