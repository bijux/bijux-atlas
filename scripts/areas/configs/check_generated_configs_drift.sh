#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

before_index="$(shasum configs/INDEX.md | awk '{print $1}')"
before_registry="$(shasum configs/config-key-registry.md | awk '{print $1}')"
before_env_contract="$(shasum configs/contracts/env.schema.json 2>/dev/null | awk '{print $1}')"
before_env_doc="$(shasum docs/_generated/env-vars.md 2>/dev/null | awk '{print $1}')"

python3 -m bijux_atlas_scripts.cli configs generate --report text >/dev/null
python3 -m bijux_atlas_scripts.cli docs generate-config-keys-doc --report text >/dev/null
python3 -m bijux_atlas_scripts.cli configs generate --report text >/dev/null
python3 -m bijux_atlas_scripts.cli docs generate-env-vars-doc --report text >/dev/null

after_index="$(shasum configs/INDEX.md | awk '{print $1}')"
after_registry="$(shasum configs/config-key-registry.md | awk '{print $1}')"
after_env_contract="$(shasum configs/contracts/env.schema.json | awk '{print $1}')"
after_env_doc="$(shasum docs/_generated/env-vars.md | awk '{print $1}')"

if [ "$before_index" != "$after_index" ] || [ "$before_registry" != "$after_registry" ] || \
   [ "$before_env_contract" != "$after_env_contract" ] || [ "$before_env_doc" != "$after_env_doc" ]; then
  echo "configs generated docs drift detected; run make configs-gen-check and commit outputs" >&2
  exit 1
fi

echo "configs generated docs drift check passed"
