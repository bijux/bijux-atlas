from __future__ import annotations

import json
import io
import sys
from contextlib import redirect_stdout
from pathlib import Path


def _iter_files(root: Path, suffix: str) -> list[Path]:
    return sorted(path for path in root.rglob(f"*{suffix}") if path.is_file())


def _max_loc(root: Path, suffix: str, err_gt: int, warn_gt: int, label: str) -> int:
    err_rows: list[str] = []
    warn_rows: list[str] = []
    for path in _iter_files(root, suffix):
        loc = len(path.read_text(encoding="utf-8", errors="ignore").splitlines())
        rel = path.as_posix()
        if loc > err_gt:
            err_rows.append(f"{loc:6d} {rel}")
        elif loc > warn_gt:
            warn_rows.append(f"{loc:6d} {rel}")
    if err_rows:
        print(f"ERROR: {label} max_loc policy violations (LOC > {err_gt}):")
        for row in sorted(err_rows):
            print(row)
        return 1
    if warn_rows:
        print(f"WARN: {label} max_loc advisory violations ({warn_gt} < LOC <= {err_gt}):")
        for row in sorted(warn_rows):
            print(row)
    else:
        print(f"INFO: {label} max_loc policy compliant.")
    return 0


def _max_depth(root: Path, suffixes: tuple[str, ...], depth_gt: int, label: str) -> int:
    offenders: list[str] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.suffix not in suffixes:
            continue
        rel = path.relative_to(root).as_posix()
        depth = rel.count("/") + 1
        if depth > depth_gt:
            offenders.append(f"{depth:3d} {path.as_posix()}")
    if offenders:
        print(f"ERROR: {label} max_depth policy violations (depth > {depth_gt}):")
        for row in offenders:
            print(row)
        return 1
    print(f"INFO: {label} max_depth policy compliant.")
    return 0


def _max_files_per_dir(root: Path, suffix: str, max_files: int, label: str, metric: str) -> int:
    counts: dict[str, int] = {}
    for path in _iter_files(root, suffix):
        key = path.parent.as_posix()
        counts[key] = counts.get(key, 0) + 1
    offenders = sorted(((count, directory) for directory, count in counts.items() if count > max_files), reverse=True)
    if offenders:
        print(f"ERROR: {label} {metric} policy violations (files > {max_files}):")
        for count, directory in offenders:
            print(f"{count:4d} {directory}")
        return 1
    print(f"INFO: {label} {metric} policy compliant.")
    return 0


def run_gate(gate: str) -> int:
    crates_root = Path("crates")
    atlas_root = Path("packages/atlasctl/src/atlasctl")
    packages_root = Path("packages")
    if gate == "culprits-crates-max_loc":
        return _max_loc(crates_root, ".rs", err_gt=1000, warn_gt=800, label="crates")
    if gate == "culprits-crates-max_depth":
        return _max_depth(crates_root, (".rs",), depth_gt=7, label="crates")
    if gate == "culprits-crates-file-max_rs_files_per_dir":
        return _max_files_per_dir(crates_root, ".rs", max_files=10, label="crates", metric="max_rs_files_per_dir")
    if gate == "culprits-crates-file-max_modules_per_dir":
        return _max_files_per_dir(crates_root, ".rs", max_files=16, label="crates", metric="max_modules_per_dir")
    if gate == "culprits-atlasctl-max_loc":
        return _max_loc(atlas_root, ".py", err_gt=1000, warn_gt=800, label="atlasctl(py)")
    if gate == "culprits-atlasctl-sh-max_loc":
        return _max_loc(atlas_root, ".sh", err_gt=1000, warn_gt=800, label="atlasctl(sh)")
    if gate == "culprits-packages-sh-max_loc":
        return _max_loc(packages_root, ".sh", err_gt=1000, warn_gt=800, label="packages(sh)")
    if gate == "culprits-atlasctl-max_depth":
        return _max_depth(atlas_root, (".py", ".json", ".md"), depth_gt=10, label="atlasctl")
    if gate == "culprits-atlasctl-file-max_py_files_per_dir":
        return _max_files_per_dir(atlas_root, ".py", max_files=15, label="atlasctl", metric="max_py_files_per_dir")
    if gate == "culprits-atlasctl-file-max_modules_per_dir":
        return _max_files_per_dir(atlas_root, ".py", max_files=15, label="atlasctl", metric="max_modules_per_dir")
    if gate == "culprits-all-crates":
        codes = [
            run_gate("culprits-crates-max_loc"),
            run_gate("culprits-crates-max_depth"),
            run_gate("culprits-crates-file-max_rs_files_per_dir"),
            run_gate("culprits-crates-file-max_modules_per_dir"),
        ]
        return 1 if any(code != 0 for code in codes) else 0
    if gate == "culprits-all-atlasctl":
        codes = [
            run_gate("culprits-atlasctl-max_loc"),
            run_gate("culprits-atlasctl-sh-max_loc"),
            run_gate("culprits-atlasctl-max_depth"),
            run_gate("culprits-atlasctl-file-max_py_files_per_dir"),
            run_gate("culprits-atlasctl-file-max_modules_per_dir"),
        ]
        return 1 if any(code != 0 for code in codes) else 0
    if gate == "culprits-all-packages-loc":
        codes = [
            run_gate("culprits-atlasctl-max_loc"),
            run_gate("culprits-atlasctl-sh-max_loc"),
            run_gate("culprits-packages-sh-max_loc"),
        ]
        return 1 if any(code != 0 for code in codes) else 0
    print(f"unknown gate: {gate}")
    return 2


def run_gate_structured(gate: str) -> dict[str, object]:
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = run_gate(gate)
    lines = [ln for ln in buf.getvalue().splitlines() if ln.strip()]
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "policy-culprits-make-gate",
        "gate": gate,
        "status": "ok" if code == 0 else ("error" if code == 1 else "invalid"),
        "exit_code": code,
        "stdout_lines": lines,
    }


def main() -> int:
    args = list(sys.argv[1:])
    as_json = False
    if "--json" in args:
        as_json = True
        args = [a for a in args if a != "--json"]
    if len(args) != 2 or args[0] != "--gate":
        print("usage: culprits_make.py [--json] --gate <gate>", file=sys.stderr)
        return 2
    if as_json:
        payload = run_gate_structured(args[1])
        print(json.dumps(payload, sort_keys=True))
        return int(payload["exit_code"])
    return run_gate(args[1])


if __name__ == "__main__":
    raise SystemExit(main())
