#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
contract = json.loads((ROOT / "docs" / "contracts" / "CHART_VALUES.json").read_text())
expected = set(contract["top_level_keys"])

values_text = (ROOT / "ops" / "k8s" / "charts" / "bijux-atlas" / "values.yaml").read_text()
actual = {
    m.group(1)
    for m in re.finditer(r"^([A-Za-z][A-Za-z0-9_]*)\s*:", values_text, flags=re.MULTILINE)
}
if expected != actual:
    print("chart values contract drift", file=sys.stderr)
    print("missing in contract:", sorted(actual - expected), file=sys.stderr)
    print("extra in contract:", sorted(expected - actual), file=sys.stderr)
    sys.exit(1)

print("chart values contract check passed")
