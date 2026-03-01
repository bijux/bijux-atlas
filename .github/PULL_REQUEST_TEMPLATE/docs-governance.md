## Docs Governance Checklist

- [ ] If docs were added, list what was merged or removed to avoid growth-only changes.
- [ ] If docs were moved, add redirects and link the canonical destination.
- [ ] If commands are documented, confirm they exist in `bijux dev atlas ...` surface.
- [ ] If make targets are documented, confirm they exist in `make help` output.
- [ ] If configs are documented, confirm they exist in `configs/inventory/consumers.json`.
- [ ] If schemas are documented, confirm they exist in `docs/_internal/generated/schema-index.json`.
- [ ] If reference pages changed, regenerate with `bijux dev atlas docs reference generate --allow-subprocess --allow-write`.
- [ ] Run `make docs-validate` and attach result or CI link.
