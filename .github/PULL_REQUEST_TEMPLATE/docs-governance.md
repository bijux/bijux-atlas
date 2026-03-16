## Docs Governance Checklist

- [ ] If docs were added, list what was merged or removed to avoid growth-only changes.
- [ ] If docs were moved or renamed, update inbound links, state the canonical destination, and run `bijux dev atlas docs redirects sync --allow-write`.
- [ ] Published docs stay within `docs/01-introduction` through `docs/08-contracts`, `docs/assets`, and `docs/index.md`; no Markdown page was added under `docs/_internal`.
- [ ] Root docs (`README.md`, `SECURITY.md`, `CONTRIBUTING.md`) and any touched crate README remain consistent with the numbered docs spine and the installed command routes.
- [ ] If commands are documented, confirm they exist in an installed surface: `bijux atlas ...` or `bijux dev atlas ...`.
- [ ] If makes targets are documented, confirm they exist in `make help` and `makes/target-list.json`.
- [ ] If configs are documented, confirm they exist in `configs/registry/inventory/consumers.json`.
- [ ] If schemas are documented, confirm they exist in the owning schema index (`configs/generated/docs/schema-index.json` or `ops/schema/generated/schema-index.json`, as appropriate).
- [ ] If reference pages changed, regenerate with `make docs-reference-regenerate` or `bijux dev atlas docs reference generate --allow-subprocess --allow-write`.
- [ ] Run `make docs-validate` and attach result or CI link.
