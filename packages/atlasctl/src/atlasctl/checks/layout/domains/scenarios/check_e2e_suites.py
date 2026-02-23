#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("ops", "packages", "configs", "makefiles")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
SCHEMA_PATH = ROOT / "ops/schema/e2e-suites.schema.json"
MANIFEST_PATH = ROOT / "ops/e2e/suites/suites.json"


def main() -> int:
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    errors: list[str] = []

    for key in schema.get("required", []):
        if key not in manifest:
            errors.append(f"missing required key: {key}")

    suite_ids: set[str] = set()
    scenario_ids: set[str] = set()
    required_suites = {"fixture-medium", "release-diff-fixture", "rollback-drill", "upgrade-drill"}
    for i, suite in enumerate(manifest.get("suites", [])):
        sid = suite.get("id")
        if not isinstance(sid, str) or re.match(r"^[a-z0-9-]+$", sid) is None:
            errors.append(f"suite[{i}] invalid id")
            continue
        if sid in suite_ids:
            errors.append(f"duplicate suite id: {sid}")
        suite_ids.add(sid)
        if not suite.get("scenarios"):
            errors.append(f"suite `{sid}` has no scenarios")

        for j, scenario in enumerate(suite.get("scenarios", [])):
            scid = scenario.get("id")
            if not isinstance(scid, str) or re.match(r"^[a-z0-9-]+$", scid) is None:
                errors.append(f"suite `{sid}` scenario[{j}] invalid id")
                continue
            key = f"{sid}:{scid}"
            if key in scenario_ids:
                errors.append(f"duplicate scenario id in suite: {key}")
            scenario_ids.add(key)

            runner = scenario.get("runner", "")
            if not isinstance(runner, str) or not runner.strip():
                errors.append(f"suite `{sid}` scenario `{scid}` missing runner")

            budget = scenario.get("budget", {})
            if not isinstance(budget, dict):
                errors.append(f"suite `{sid}` scenario `{scid}` invalid budget")
                continue
            for req in ("expected_time_seconds", "expected_qps", "expected_artifacts"):
                if req not in budget:
                    errors.append(f"suite `{sid}` scenario `{scid}` budget missing `{req}`")
            artifacts = budget.get("expected_artifacts", [])
            if isinstance(artifacts, list):
                for artifact in artifacts:
                    if isinstance(artifact, str) and any(ch.isdigit() for ch in artifact.split("/")[-1]) and "report" in artifact and "run-id" not in artifact:
                        errors.append(f"suite `{sid}` scenario `{scid}` suspicious unstable report artifact name: {artifact}")
            if sid in {"rollback-drill", "upgrade-drill"} and isinstance(artifacts, list):
                if not any(isinstance(a, str) and a.endswith(".json") and "report" in a for a in artifacts):
                    errors.append(f"suite `{sid}` scenario `{scid}` must declare a json report artifact")
                if not any(isinstance(a, str) and a.endswith(".md") and "report" in a for a in artifacts):
                    errors.append(f"suite `{sid}` scenario `{scid}` must declare a markdown report artifact")

    for suite_id in sorted(required_suites):
        if suite_id not in suite_ids:
            errors.append(f"required e2e suite missing: {suite_id}")

    if errors:
        print("e2e suites contract failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("e2e suites contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
