# Documentation Ownership Policy

- Owner: `docs-governance`
- Stability: `stable`

## Policy

Every maintained documentation area has an explicit owner and review boundary.

## Ownership Map

- `docs/product/`: `bijux-atlas-product`
- `docs/operations/`: `bijux-atlas-operations`
- `docs/contracts/`: `docs-governance`
- `docs/reference/`: `docs-governance`
- `docs/development/`: `docs-governance`
- `crates/*/docs/`: crate owners in `CODEOWNERS`

## No Anonymous Docs Rule

- Every new canonical documentation page must declare ownership metadata.
- Ownership must be discoverable via either:
- `- Owner: <owner-id>` in the page header, or
- an entry in `docs/metadata/front-matter.index.json`.

## Verification

```bash
make check
```
