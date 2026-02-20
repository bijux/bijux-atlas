#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -euo pipefail

TAG="${1:-}"
if [[ -z "$TAG" ]]; then
  echo "usage: $0 <tag>" >&2
  exit 1
fi

ATLAS_VERSION="${TAG#v}"
UMB_RANGE=">=${ATLAS_VERSION},<$(echo "$ATLAS_VERSION" | awk -F. '{printf "%d.%d.0", $1, $2+1}')"

OUT="docs/reference/compatibility/umbrella-atlas-matrix.md"
cat > "$OUT" <<DOC
# Compatibility Matrix: bijux Umbrella <-> bijux-atlas

| bijux umbrella | bijux-atlas plugin | status | notes |
|---|---|---|---|
| \`${ATLAS_VERSION%%.*}.x\` | \`${ATLAS_VERSION}\` | supported | plugin advertises \`compatible_umbrella: ${UMB_RANGE}\` |
| future major | \`${ATLAS_VERSION}\` | unsupported | plugin handshake must fail compatibility check |

## Validation Rule

The umbrella validates plugin metadata range against umbrella semver before dispatch.
If incompatible, the umbrella returns a structured machine error and does not execute plugin commands.
DOC