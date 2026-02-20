#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
from pathlib import Path

manifest = json.loads(Path("ops/k8s/tests/manifest.json").read_text())
ownership = json.loads(Path("ops/k8s/tests/ownership.json").read_text())

tests = manifest["tests"]
owners = ownership["owners"]

errors = []
all_scripts = {t["script"] for t in tests}
scripts_by_name = {}
for script in all_scripts:
    scripts_by_name.setdefault(Path(script).name, []).append(script)

for t in tests:
    if "owner" not in t:
        errors.append(f"missing owner in manifest: {t['script']}")
    if "timeout_seconds" not in t:
        errors.append(f"missing timeout_seconds in manifest: {t['script']}")
    groups = t.get("groups")
    if not isinstance(groups, list) or not groups:
        errors.append(f"missing/non-list groups in manifest: {t['script']}")
    efm = t.get("expected_failure_modes")
    if not isinstance(efm, list) or not efm:
        errors.append(f"missing/non-list expected_failure_modes in manifest: {t['script']}")
    if groups != sorted(groups or []):
        errors.append(f"manifest groups must be sorted for deterministic ordering: {t['script']}")

    # Failure-mode contract: any explicit failure_mode emitted by script must be declared in expected_failure_modes.
    script_path = Path("ops/k8s/tests") / t["script"]
    if script_path.exists():
        body = script_path.read_text(encoding="utf-8")
        emitted = {m.lower() for m in re.findall(r"failure_mode\\s*[:=]\\s*([a-z0-9_]+)", body, flags=re.IGNORECASE)}
        declared = {m.lower() for m in t.get("expected_failure_modes", []) if isinstance(m, str)}
        undeclared = sorted(emitted - declared)
        if undeclared:
            errors.append(f"script emits undeclared failure_mode(s) {undeclared} for {t['script']}")

claimed = set()
for owner, scripts in owners.items():
    for s in scripts:
        resolved = s
        if s not in all_scripts and "/" not in s:
            matches = scripts_by_name.get(s, [])
            if len(matches) == 1:
                resolved = matches[0]
            elif len(matches) > 1:
                errors.append(f"ownership map test '{s}' is ambiguous for owner '{owner}': {matches}")
                continue
        claimed.add(resolved)
        if resolved not in all_scripts:
            errors.append(f"ownership map has unknown test '{s}' for owner '{owner}'")

for s in sorted(all_scripts):
    if s not in claimed:
        errors.append(f"manifest test not in ownership map: {s}")

for t in tests:
    if t["owner"] not in owners:
        errors.append(f"manifest owner '{t['owner']}' not declared in ownership map for {t['script']}")

if errors:
    for e in errors:
        print(e)
    raise SystemExit(1)

print("k8s test contract check passed")
