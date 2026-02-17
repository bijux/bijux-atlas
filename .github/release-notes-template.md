## Bijux Atlas Release {{tag}}

### Summary
- Release date: {{date}}
- Commit: {{commit}}
- Container image: `ghcr.io/bijux/bijux-atlas:{{tag}}`

### Highlights
- 

### Compatibility
- See `docs/reference/compatibility/umbrella-atlas-matrix.md`.

### Supply Chain Artifacts
- SBOM: SPDX JSON artifact attached to release workflow.
- Signature: cosign signature published for image digest.

### Verification
```bash
cosign verify \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  ghcr.io/bijux/bijux-atlas:{{tag}}
```
