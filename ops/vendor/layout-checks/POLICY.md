# Shell Probe Policy

- Shell probes are quarantined under `ops/vendor/layout-checks/`.
- Shell probes must include:
  - `#!/usr/bin/env bash`
  - `set -euo pipefail`
- Shell probes must not call `python` directly; invoke `atlasctl` entrypoints instead.
- Shell probes must not use `curl`/`wget` unless explicitly allowlisted.
- Shell probes must not write repository files directly.
