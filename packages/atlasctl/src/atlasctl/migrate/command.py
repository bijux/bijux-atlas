from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from datetime import date
from pathlib import Path

from ..checks.repo.legacy_native import check_layout_contract
from ..core.context import RunContext
from ..exit_codes import ERR_CONTRACT, ERR_USER
from ..inventory.command import collect_commands, collect_legacy_scripts


@dataclass(frozen=True)
class MigrationException:
    id: str
    owner: str
    expiry: str
    reason: str
    allow_remaining: int
    path_glob: str | None = None

    def is_expired(self) -> bool:
        return date.fromisoformat(self.expiry) < date.today()


def _run(repo_root: Path, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, check=False)
    return proc.returncode


_PATH_REPLACEMENTS: tuple[tuple[re.Pattern[str], str], ...] = (
    (re.compile(r"\./charts/"), "./ops/k8s/charts/"),
    (re.compile(r"\./e2e/"), "./ops/e2e/"),
    (re.compile(r"\./load/"), "./ops/load/"),
    (re.compile(r"\./observability/"), "./ops/obs/"),
    (re.compile(r"\./datasets/"), "./ops/datasets/"),
    (re.compile(r"\./fixtures/"), "./ops/fixtures/"),
    (re.compile(r"docs/operations/ops/"), "docs/operations/"),
    (re.compile(r"operations/ops/"), "operations/"),
)


def _tracked_files(repo_root: Path) -> list[Path]:
    proc = subprocess.run(
        ["git", "ls-files", "Makefile", "makefiles", "scripts", ".github", "docs", "ops"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return []
    return [repo_root / line for line in proc.stdout.splitlines() if line.strip()]


def _replace_legacy_paths(repo_root: Path) -> int:
    changed = 0
    for file_path in _tracked_files(repo_root):
        if not file_path.is_file():
            continue
        try:
            before = file_path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        after = before
        for pattern, repl in _PATH_REPLACEMENTS:
            after = pattern.sub(repl, after)
        if after != before:
            file_path.write_text(after, encoding="utf-8")
            changed += 1
    return changed


def _prune_legacy_root_aliases(repo_root: Path) -> None:
    for legacy in ("charts", "e2e", "load", "observability", "datasets", "fixtures"):
        path = repo_root / legacy
        if path.is_symlink() or path.is_file():
            path.unlink(missing_ok=True)
            continue
        if path.is_dir():
            try:
                path.rmdir()
            except OSError:
                pass


def _migration_exceptions(repo_root: Path) -> list[MigrationException]:
    path = repo_root / "configs/policy/migration_exceptions.json"
    if not path.exists():
        return []
    payload = json.loads(path.read_text(encoding="utf-8"))
    out: list[MigrationException] = []
    for row in payload.get("exceptions", []):
        out.append(
            MigrationException(
                id=str(row["id"]),
                owner=str(row["owner"]),
                expiry=str(row["expiry"]),
                reason=str(row["reason"]),
                allow_remaining=int(row.get("allow_remaining", 0)),
                path_glob=row.get("path_glob"),
            )
        )
    return out


def _map_entries(repo_root: Path) -> list[dict[str, str]]:
    path = repo_root / "packages/atlasctl/docs/migration/map.md"
    if not path.exists():
        return []
    lines = path.read_text(encoding="utf-8").splitlines()
    out: list[dict[str, str]] = []
    for line in lines:
        if not line.startswith("| `"):
            continue
        parts = [part.strip() for part in line.strip().strip("|").split("|")]
        if len(parts) < 2:
            continue
        legacy = parts[0].strip("`")
        module = parts[1].strip("`")
        out.append({"legacy": legacy, "module": module})
    return out


def _migration_status_payload(repo_root: Path) -> dict[str, object]:
    legacy = collect_legacy_scripts(repo_root).get("files", [])
    map_entries = _map_entries(repo_root)
    mapped = {row["legacy"] for row in map_entries}
    blocked_map = {
        row["legacy"]: "migration.todo entry in docs/migration/map.md"
        for row in map_entries
        if "migration.todo" in row["module"]
    }
    migrated = sorted(path for path in legacy if path in mapped and path not in blocked_map)
    remaining = sorted(path for path in legacy if path not in mapped)
    blocked = [{"path": p, "reason": blocked_map[p]} for p in sorted(blocked_map.keys()) if p in legacy]
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "total_legacy_scripts": len(legacy),
        "migrated": len(migrated),
        "remaining": len(remaining),
        "blocked": blocked,
        "remaining_paths": remaining,
        "command_count": collect_commands(repo_root).get("count", 0),
    }
    return payload


def _active_allowance(repo_root: Path) -> tuple[int, list[str], list[str]]:
    exceptions = _migration_exceptions(repo_root)
    expired = [f"{x.id} expired on {x.expiry} ({x.owner})" for x in exceptions if x.is_expired()]
    active = [x for x in exceptions if not x.is_expired()]
    allow = max([0, *[x.allow_remaining for x in active]])
    active_desc = [f"{x.id}: allow_remaining={x.allow_remaining} owner={x.owner}" for x in active]
    return allow, active_desc, expired


def _status(ns: argparse.Namespace, repo_root: Path) -> int:
    payload = _migration_status_payload(repo_root)
    allow_default, active_exc, expired_exc = _active_allowance(repo_root)
    payload["allow_remaining_default"] = allow_default
    payload["active_exceptions"] = active_exc
    payload["expired_exceptions"] = expired_exc
    print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
    return 0 if not expired_exc else ERR_CONTRACT


def _gate(ns: argparse.Namespace, repo_root: Path) -> int:
    payload = _migration_status_payload(repo_root)
    allow_default, active_exc, expired_exc = _active_allowance(repo_root)
    allow = int(ns.allow_remaining) if ns.allow_remaining is not None else allow_default
    payload["allow_remaining"] = allow
    payload["active_exceptions"] = active_exc
    payload["expired_exceptions"] = expired_exc
    if expired_exc:
        payload["status"] = "fail"
        payload["reason"] = "expired migration exceptions"
        print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return ERR_CONTRACT
    if int(payload["remaining"]) > allow:
        payload["status"] = "fail"
        payload["reason"] = f"remaining scripts exceed allowance ({payload['remaining']} > {allow})"
        print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return ERR_CONTRACT
    print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
    return 0


def _diff(ns: argparse.Namespace, ctx: RunContext) -> int:
    current = sorted(collect_legacy_scripts(ctx.repo_root).get("files", []))
    state = ctx.scripts_root / "migration-state.json"
    previous: list[str] = []
    if state.exists():
        try:
            previous = sorted(json.loads(state.read_text(encoding="utf-8")).get("files", []))
        except Exception:
            previous = []
    removed = sorted(set(previous) - set(current))
    added = sorted(set(current) - set(previous))
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "removed_since_last": removed,
        "added_since_last": added,
        "current_count": len(current),
        "previous_count": len(previous),
    }
    state.parent.mkdir(parents=True, exist_ok=True)
    state.write_text(json.dumps({"files": current}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
    return 0


def run_migrate_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.migrate_cmd == "layout":
        changed = _replace_legacy_paths(ctx.repo_root)
        _prune_legacy_root_aliases(ctx.repo_root)
        code, errors = check_layout_contract(ctx.repo_root)
        if code != 0:
            print("layout migration check failed:")
            for err in errors[:100]:
                print(f"- {err}")
            return code
        print(f"layout migration completed (updated_files={changed})")
        return 0
    if ns.migrate_cmd == "status":
        return _status(ns, ctx.repo_root)
    if ns.migrate_cmd == "gate":
        return _gate(ns, ctx.repo_root)
    if ns.migrate_cmd == "diff":
        return _diff(ns, ctx)
    return ERR_USER


def configure_migrate_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("migration", help="migration tracking and enforcement commands")
    p.add_argument("--json", action="store_true", help="emit JSON output")
    msub = p.add_subparsers(dest="migrate_cmd", required=True)
    msub.add_parser("layout", help="apply deterministic layout path migrations")
    msub.add_parser("status", help="print migration status for scripts removal")
    gate = msub.add_parser("gate", help="enforce migration gate with allowance policy")
    gate.add_argument("--allow-remaining", type=int, help="temporary remaining budget override")
    msub.add_parser("diff", help="show removed legacy files since previous run")
