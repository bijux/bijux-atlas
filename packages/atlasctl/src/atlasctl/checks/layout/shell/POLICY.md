# Shell Policy

This directory is transitional and must shrink over time.

Rules:

- Use `#!/usr/bin/env bash` and `set -euo pipefail`.
- Do not call `python` directly from shell scripts.
- Do not call `curl`/`wget` unless explicitly annotated with `shell-allow-network-fetch`.
- Do not write repository files directly from shell checks.
- Shell execution from Python must use `atlasctl.core.exec`/`atlasctl.core.exec_shell`.

Long-term target: near-zero shell checks under this directory.
