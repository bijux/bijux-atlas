#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
contract = json.loads((ROOT / "docs/contracts/CONFIG_KEYS.json").read_text())
allowed = set(contract["env_keys"])

key_pattern = re.compile(r'"([A-Z][A-Z0-9_]+)"')
call_pattern = re.compile(
    r"(?:std::env::var|env::var|env_bool|env_u64|env_usize|env_f64|env_duration_ms|env_list|env_map|env_dataset_list)\(\s*\"([A-Z][A-Z0-9_]+)\""
)

used = set()
violations = []
for path in ROOT.joinpath("crates").rglob("*.rs"):
    text = path.read_text()
    for m in call_pattern.finditer(text):
        key = m.group(1)
        used.add(key)
        if key not in allowed:
            violations.append(f"{path}: undeclared env key {key}")

# disallow ad-hoc env::var outside explicit config entry points
allowed_env_read_paths = {
    "crates/bijux-atlas-server/src/main.rs",
    "crates/bijux-atlas-core/src/lib.rs",
    "crates/bijux-atlas-cli/src/atlas_command_actions.rs",
    "crates/bijux-atlas-cli/src/atlas_command_actions/ingest_inputs.rs",
    "crates/bijux-atlas-cli/src/artifact_validation.rs",
    "crates/bijux-atlas-cli/src/artifact_validation/gc.rs",
    "crates/bijux-atlas-cli/src/lib.rs",
    "crates/bijux-atlas-query/benches/query_patterns.rs",
    "crates/bijux-atlas-server/tests/redis_optional.rs",
}
for path in ROOT.joinpath("crates").rglob("*.rs"):
    rel = str(path.relative_to(ROOT))
    text = path.read_text()
    if "env::var(" in text or "std::env::var(" in text:
        if rel not in allowed_env_read_paths:
            violations.append(f"{rel}: ad-hoc env var read is forbidden")

missing = sorted(allowed - used)
if missing:
    # allow core platform keys even if they are not always read in code paths
    tolerated = {"HOME", "HOSTNAME", "XDG_CACHE_HOME", "XDG_CONFIG_HOME", "REDIS_URL"}
    hard_missing = [k for k in missing if k not in tolerated]
    if hard_missing:
        violations.append("declared env keys not used: " + ", ".join(hard_missing))

if violations:
    print("config keys contract check failed", file=sys.stderr)
    for v in violations:
        print(f"- {v}", file=sys.stderr)
    sys.exit(1)

print("config keys contract check passed")
