#!/usr/bin/env sh
set -eu

python3 - <<'PY'
import json
from pathlib import Path

cfg = Path("configs/policy/policy.json")
schema = Path("configs/policy/policy.schema.json")

if not cfg.exists() or not schema.exists():
    raise SystemExit("missing policy schema/config")

cfg_data = json.loads(cfg.read_text())
required = {"schema_version", "allow_override", "network_in_unit_tests"}
missing = sorted(required - set(cfg_data.keys()))
if missing:
    raise SystemExit(f"missing required config keys: {missing}")

if cfg_data["allow_override"] is not False:
    raise SystemExit("allow_override must be false")
if cfg_data["network_in_unit_tests"] is not False:
    raise SystemExit("network_in_unit_tests must be false")
print("policy config validated")
PY

./scripts/require-crate-docs.sh
./scripts/no-network-unit-tests.sh
