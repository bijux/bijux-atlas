#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
contract = json.loads((ROOT / "docs" / "contracts" / "CLI_COMMANDS.json").read_text())
expected = contract["commands"]

doc_list = (
    ROOT / "crates" / "bijux-atlas-cli" / "docs" / "CLI_COMMAND_LIST.md"
).read_text().splitlines()

if expected != doc_list:
    print("CLI command list drift: docs/contracts/CLI_COMMANDS.json != CLI_COMMAND_LIST.md", file=sys.stderr)
    sys.exit(1)

print("cli SSOT check passed")