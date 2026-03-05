#!/usr/bin/env bash
set -euo pipefail

cat <<'TEXT'
Add one of these lines to your shell startup file:

bash:
  source ~/.local/share/bijux-dev-atlas/completions/bijux-dev-atlas.bash

zsh:
  fpath=(~/.local/share/bijux-dev-atlas/completions $fpath)
  autoload -Uz compinit && compinit

fish:
  cp ~/.local/share/bijux-dev-atlas/completions/bijux-dev-atlas.fish ~/.config/fish/completions/
TEXT
