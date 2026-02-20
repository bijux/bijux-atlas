#!/usr/bin/env python3
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUDGETS_PATH = ROOT / "configs/layout/dir-budgets.json"


def _count_top_level_files(path: Path, ignore_marker: str | None) -> int:
    count = 0
    for p in path.iterdir():
        if not p.is_file():
            continue
        if ignore_marker:
            try:
                if ignore_marker in p.read_text(encoding="utf-8", errors="ignore"):
                    continue
            except OSError:
                pass
        count += 1
    return count


def _count_top_level_dirs(path: Path) -> int:
    return sum(1 for p in path.iterdir() if p.is_dir())


def main() -> int:
    try:
        data = json.loads(BUDGETS_PATH.read_text(encoding="utf-8"))
    except OSError as exc:
        print(f"layout budget check failed: cannot read {BUDGETS_PATH}: {exc}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as exc:
        print(f"layout budget check failed: invalid json in {BUDGETS_PATH}: {exc}", file=sys.stderr)
        return 1

    failures: list[str] = []
    explain: list[tuple[str, str, int, int]] = []

    for entry in data.get("budgets", []):
        rel = entry["path"]
        path = ROOT / rel
        if not path.exists() or not path.is_dir():
            failures.append(f"- {rel}: path does not exist")
            continue

        ignore_marker = entry.get("ignore_shim_marker")

        if "max_top_level_files" in entry:
            actual = _count_top_level_files(path, ignore_marker)
            limit = int(entry["max_top_level_files"])
            explain.append((rel, "top_level_files", actual, limit))
            if actual > limit:
                failures.append(f"- {rel}: top-level files {actual} > {limit}")

        if "max_top_level_dirs" in entry:
            actual = _count_top_level_dirs(path)
            limit = int(entry["max_top_level_dirs"])
            explain.append((rel, "top_level_dirs", actual, limit))
            if actual > limit:
                failures.append(f"- {rel}: top-level dirs {actual} > {limit}")

    if failures:
        print("directory budget check failed:", file=sys.stderr)
        for line in failures:
            print(line, file=sys.stderr)
        print("\nTop offenders:", file=sys.stderr)
        over = [row for row in explain if row[2] > row[3]]
        over.sort(key=lambda r: (r[2] - r[3]), reverse=True)
        for rel, metric, actual, limit in over[:10]:
            print(f"- {rel} {metric}: {actual} (limit {limit}, over +{actual - limit})", file=sys.stderr)
        return 1

    print("directory budgets passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
