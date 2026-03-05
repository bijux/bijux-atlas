#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPLETIONS_DIR="$ROOT_DIR/tools/cli/completions"

mkdir -p "$HOME/.local/share/bijux-dev-atlas/completions"
cp "$COMPLETIONS_DIR/bijux-dev-atlas.bash" "$HOME/.local/share/bijux-dev-atlas/completions/"
cp "$COMPLETIONS_DIR/_bijux-dev-atlas" "$HOME/.local/share/bijux-dev-atlas/completions/"
cp "$COMPLETIONS_DIR/bijux-dev-atlas.fish" "$HOME/.local/share/bijux-dev-atlas/completions/"

echo "installed completions into $HOME/.local/share/bijux-dev-atlas/completions"
