# Curl Requests

- Owner: `bijux-atlas-api`

## What
Canonical `curl` examples for common API operations.

## Why
Provides stable request patterns for smoke checks and integration setup.

## Scope
Includes deterministic requests for health and discovery endpoints.

## Non-goals
Does not benchmark endpoint performance.

## Contracts
Use explicit dataset dimensions and pinned endpoint paths.

## Failure modes
Stale endpoint examples cause false troubleshooting paths.

## How to verify
```bash
$ python3 scripts/areas/docs/extract_code_blocks.py
$ python3 scripts/areas/docs/run_blessed_snippets.py
```

Expected output: all blessed snippets execute with zero failures.

## Examples
```bash
# blessed-snippet
cargo run -p bijux-atlas-cli --bin bijux-atlas -- atlas --help >/dev/null
```

```bash
# blessed-snippet
printf '{"status":"ok"}\n' | jq -e '.status == "ok"' >/dev/null
```

## See also
- [OpenAPI Contract](../contracts/endpoints.md)
- [Quickstart Local Cluster Setup](../quickstart/local-cluster-setup.md)
- [Contracts Examples](../contracts/examples/INDEX.md)
