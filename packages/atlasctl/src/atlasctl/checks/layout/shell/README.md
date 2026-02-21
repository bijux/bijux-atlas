# layout shell checks

This directory contains transitional shell-based layout checks.

Policy:
- New policy-critical checks should be implemented in Python modules under `checks/layout/*`.
- Shell checks must be invoked via `atlasctl core.exec` boundaries (no ad-hoc direct calls from Python modules).
- Shell files here must keep `#!/usr/bin/env bash` and `set -euo pipefail`.

Migration plan:
- Root-shape and forbidden-path checks are being migrated to Python first.
- Remaining shell checks are tracked for gradual conversion as domain checks gain parity.
