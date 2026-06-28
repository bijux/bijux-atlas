## Bijux Atlas Release {{tag}}

### Summary
- Release date: {{date}}
- Commit: {{commit}}
- Release type: {{release_type}}
- Published crates: `bijux-atlas`, `bijux-atlas-api`, `bijux-atlas-cli`, `bijux-atlas-core`, `bijux-atlas-ingest`, `bijux-atlas-model`, `bijux-atlas-ops`, `bijux-atlas-query`, `bijux-atlas-runtime`, `bijux-atlas-server`, `bijux-atlas-store`
- Repository-only crate: `bijux-atlas-dev`
- Direct commands: `bijux-atlas`, `bijux-atlas-server`, `bijux-atlas-openapi`
- Container image: `ghcr.io/bijux/bijux-atlas/atlas-runtime:{{tag}}`

### Highlights
- 

### Compatibility
- Reader contract: `docs/bijux-atlas/contracts/api-compatibility.md`
- Release workflow: `docs/bijux-atlas-dev/delivery/release-and-versioning.md`
- Call out any deprecated, redirected, or removed surface explicitly.

### Install or Upgrade
```bash
cargo install --locked bijux-atlas-cli --bin bijux-atlas
cargo install --locked bijux-atlas-server --bin bijux-atlas-server
cargo install --locked bijux-atlas-api --bin bijux-atlas-openapi
```

### Verify
```bash
bijux-atlas version
bijux-atlas-server --help
bijux-atlas-openapi --help
```

### Supply Chain Artifacts
- SBOM: SPDX JSON artifact attached to release workflow.
- Signature: cosign signature published for image digest.

### Signature Verification
```bash
cosign verify \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  ghcr.io/bijux/bijux-atlas/atlas-runtime:{{tag}}
```
