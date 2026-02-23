#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("ops", "packages", "configs", "makefiles")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def _load_json(rel: str) -> object:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def main() -> int:
    errors: list[str] = []

    legacy_result = _load_json("ops/load/contracts/result-schema.json")
    atlasctl_result = _load_json("packages/atlasctl/src/atlasctl/commands/ops/load/contracts/result-schema.json")
    if legacy_result != atlasctl_result:
        errors.append("ops/load/contracts/result-schema.json must mirror atlasctl ops load contracts/result-schema.json")

    updater_src = (ROOT / "packages/atlasctl/src/atlasctl/commands/ops/load/baseline/update_baseline_entrypoint.py").read_text(
        encoding="utf-8"
    )
    for token in ("--i-know-what-im-doing", "--justification"):
        if token not in updater_src:
            errors.append(f"load baseline update entrypoint missing required policy flag: {token}")

    workflow_texts = []
    for wf in sorted((ROOT / ".github/workflows").glob("*.yml")):
        workflow_texts.append(wf.read_text(encoding="utf-8"))
    combined = "\n".join(workflow_texts)
    if "ops.load.regression" not in combined and "check_regression.py" not in combined:
        errors.append("CI must run a dedicated load regression lane/check")

    if errors:
        print("ops load phase4 contract checks failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("ops load phase4 contract checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
