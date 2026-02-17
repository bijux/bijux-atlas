#!/usr/bin/env sh
set -eu

# Purpose: Render Mermaid/PlantUML diagram sources in docs/_assets/diagrams.
# Inputs: docs/_assets/diagrams/*.mmd and/or *.puml
# Outputs: docs/_assets/diagrams/*.svg for each renderable source

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
DIAGRAM_DIR="$ROOT_DIR/docs/_assets/diagrams"

rendered=0

if command -v mmdc >/dev/null 2>&1; then
  find "$DIAGRAM_DIR" -type f -name '*.mmd' | while IFS= read -r src; do
    out="${src%.mmd}.svg"
    mmdc -i "$src" -o "$out" >/dev/null
    rendered=$((rendered + 1))
  done
else
  echo "mmdc not found; skipping Mermaid rendering" >&2
fi

if command -v plantuml >/dev/null 2>&1; then
  find "$DIAGRAM_DIR" -type f -name '*.puml' | while IFS= read -r src; do
    plantuml -tsvg "$src" >/dev/null
    rendered=$((rendered + 1))
  done
else
  echo "plantuml not found; skipping PlantUML rendering" >&2
fi

if [ "$rendered" -eq 0 ]; then
  echo "diagram render check completed (no renderer available or no sources)"
else
  echo "rendered $rendered diagram(s)"
fi
