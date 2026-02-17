#!/usr/bin/env bash
# Purpose: optional local installer for ops/dev tooling.
# Inputs: --yes to proceed non-interactively.
# Outputs: installs tools when brew/cargo are available.
set -euo pipefail

ASSUME_YES=0
if [ "${1:-}" = "--yes" ]; then
  ASSUME_YES=1
fi

confirm() {
  if [ "$ASSUME_YES" -eq 1 ]; then
    return 0
  fi
  printf '%s [y/N]: ' "$1"
  read -r ans
  [ "$ans" = "y" ] || [ "$ans" = "Y" ]
}

if command -v brew >/dev/null 2>&1; then
  if confirm "Install kind, kubectl, helm, k6 via brew if missing?"; then
    command -v kind >/dev/null 2>&1 || brew install kind
    command -v kubectl >/dev/null 2>&1 || brew install kubectl
    command -v helm >/dev/null 2>&1 || brew install helm
    command -v k6 >/dev/null 2>&1 || brew install k6
  fi
else
  echo "brew not found; skipping brew-managed tools"
fi

if command -v cargo >/dev/null 2>&1; then
  if confirm "Install cargo-nextest, cargo-deny, cargo-audit, cargo-llvm-cov if missing?"; then
    cargo nextest --version >/dev/null 2>&1 || cargo install cargo-nextest --locked
    cargo deny --version >/dev/null 2>&1 || cargo install cargo-deny --locked
    cargo audit --version >/dev/null 2>&1 || cargo install cargo-audit --locked
    cargo llvm-cov --version >/dev/null 2>&1 || cargo install cargo-llvm-cov --locked
  fi
fi

echo "tool bootstrap complete"
