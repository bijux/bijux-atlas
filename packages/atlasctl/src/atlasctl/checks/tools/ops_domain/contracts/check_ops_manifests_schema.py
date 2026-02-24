from __future__ import annotations

import json
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
def run() -> int:
    from atlasctl.contracts.schema.validate import validate

    manifests_root = ROOT / "ops" / "manifests"
    errors: list[str] = []
    if not manifests_root.exists():
        print("missing ops manifests root: ops/manifests")
        return 1

    files = sorted(
        [
            *manifests_root.rglob("*.json"),
            *manifests_root.rglob("*.yaml"),
            *manifests_root.rglob("*.yml"),
        ]
    )
    if not files:
        print("no ops manifests found under ops/manifests")
        return 1
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        try:
            if path.suffix.lower() == ".json":
                payload = json.loads(path.read_text(encoding="utf-8"))
            else:
                try:
                    import yaml  # type: ignore
                except ModuleNotFoundError as exc:
                    raise RuntimeError("PyYAML is required for yaml ops manifests") from exc
                payload = yaml.safe_load(path.read_text(encoding="utf-8"))
            if not isinstance(payload, dict):
                raise RuntimeError("manifest payload must be an object")
            validate("atlasctl.ops.manifest.v1", payload)
        except Exception as exc:
            errors.append(f"{rel}: {exc}")
    if errors:
        for err in errors:
            print(err)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(run())
