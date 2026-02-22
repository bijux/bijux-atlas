from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)")


def _recipes(path: Path) -> dict[str, list[str]]:
    current = ""
    out: dict[str, list[str]] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        match = TARGET_RE.match(line)
        if match and not line.startswith("."):
            current = match.group(1)
            out.setdefault(current, [])
            continue
        if line.startswith("\t") and current:
            out[current].append(line.strip())
    return out


def test_wrapper_makefiles_delegate_once_to_atlasctl() -> None:
    for rel in ("makefiles/dev.mk", "makefiles/ci.mk"):
        recipes = _recipes(ROOT / rel)
        for target, lines in recipes.items():
            assert len(lines) == 1, f"{rel}:{target} must have exactly one recipe line"
            line = lines[0]
            assert line.startswith("@./bin/atlasctl "), f"{rel}:{target} must delegate through ./bin/atlasctl"
            assert line.count("./bin/atlasctl") == 1, f"{rel}:{target} must call atlasctl exactly once"


def test_core_wrapper_targets_use_expected_atlasctl_args() -> None:
    dev = _recipes(ROOT / "makefiles/dev.mk")
    ci = _recipes(ROOT / "makefiles/ci.mk")
    assert dev["fmt"] == ["@./bin/atlasctl dev fmt"]
    assert dev["lint"] == ["@./bin/atlasctl dev lint"]
    assert dev["test"] == ["@./bin/atlasctl dev test"]
    assert dev["test-all"] == ["@./bin/atlasctl dev test --all"]
    assert dev["check"] == ["@./bin/atlasctl dev check"]
    assert dev["atlasctl-check"] == ["@./bin/atlasctl check run --group repo"]
    assert ci["ci"] == ["@./bin/atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci"]
    assert ci["ci-all"] == ["@./bin/atlasctl ci all --json"]
    assert ci["ci-init"] == ["@./bin/atlasctl ci init --json"]
    assert ci["ci-help"] == ["@./bin/atlasctl help ci"]
