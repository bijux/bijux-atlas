# Docs change process

Use this process for all doc updates.

1. Edit the smallest set of pages that solves the problem.
2. Update redirects in `docs/redirects.json` when moving or deleting pages.
3. Regenerate docs artifacts and indexes.
4. Run docs checks.
5. Submit with a clear Conventional Commit message.

## Enforcement commands

```bash
make docs-generate
make docs-check
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
```
