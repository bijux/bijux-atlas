## Bijux Atlas Release {{tag}}

### Summary
- Release date: {{date}}
- Commit: {{commit}}
- Release type: {{release_type}}
- Crates: `bijux-atlas`, `bijux-dev-atlas`
- Installed commands: `bijux atlas ...`, `bijux dev atlas ...`
- Container image: `ghcr.io/bijux/bijux-atlas:{{tag}}`

### Highlights
- 

### Compatibility
- Reader contract: `docs/08-contracts/api-compatibility.md`
- Release workflow: `docs/06-development/release-and-versioning.md`
- Call out any deprecated, redirected, or removed surface explicitly.

### Install or Upgrade
```bash
bijux install bijux-atlas
bijux install bijux-dev-atlas

# or

cargo install bijux-atlas
cargo install bijux-dev-atlas
```

### Supply Chain Artifacts
- SBOM: SPDX JSON artifact attached to release workflow.
- Signature: cosign signature published for image digest.

### Verification
```bash
cosign verify \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  ghcr.io/bijux/bijux-atlas:{{tag}}
```
