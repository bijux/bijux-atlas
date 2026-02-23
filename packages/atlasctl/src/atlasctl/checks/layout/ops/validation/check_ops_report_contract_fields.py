from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
WRAPPERS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops/orchestrate/_wrappers.py"
ORCH = ROOT / "packages/atlasctl/src/atlasctl/commands/ops/orchestrate/command.py"
TOOLS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops/tools.py"


def main() -> int:
    errs: list[str] = []
    wrappers = WRAPPERS.read_text(encoding="utf-8", errors="ignore")
    orch = ORCH.read_text(encoding="utf-8", errors="ignore")
    tools = TOOLS.read_text(encoding="utf-8", errors="ignore")

    for token in ('"run.log"', '"report.json"', '"artifact-index.json"'):
        if token not in wrappers:
            errs.append(f"wrapper reports must use deterministic artifact names; missing {token}")
    if "strftime(" in wrappers:
        errs.append("wrapper reports must not generate timestamped artifact filenames")

    required_detail_tokens = [
        '"inputs"',
        '"inputs_hash"',
        '"config_hash"',
        '"environment_summary"',
        '"tool_versions"',
        '"command_rendered"',
        '"timings"',
    ]
    for token in required_detail_tokens:
        if token not in wrappers and token not in orch:
            errs.append(f"ops report contract token missing from orchestrate emitters: {token}")

    if '"tool_versions"' not in tools:
        errs.append("ops.tools.environment_summary must include tool_versions")

    if errs:
        print("ops report contract fields check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("ops report contract fields check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
