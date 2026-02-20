#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import fnmatch
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ALLOWLIST = ROOT / "configs/policy/layer-live-diff-allowlist.json"


def _load_allowlist() -> tuple[list[dict[str, str]], list[str]]:
    today = dt.date.today()
    payload = json.loads(ALLOWLIST.read_text(encoding="utf-8"))
    active: list[dict[str, str]] = []
    errors: list[str] = []
    for entry in payload.get("exceptions", []):
        expiry_raw = str(entry.get("expiry", ""))
        try:
            expiry = dt.date.fromisoformat(expiry_raw)
        except ValueError:
            errors.append(f"{entry.get('id', '<missing-id>')}: invalid expiry `{expiry_raw}`")
            continue
        if expiry < today:
            errors.append(f"{entry.get('id', '<missing-id>')}: expired on {expiry_raw}")
            continue
        active.append(entry)
    return active, errors


def _is_allowlisted(diff_key: str, entries: list[dict[str, str]]) -> str | None:
    for entry in entries:
        if fnmatch.fnmatch(diff_key, str(entry.get("diff_key", ""))):
            return str(entry.get("id", ""))
    return None


def _service_ports(svc: dict) -> dict[str, int]:
    out: dict[str, int] = {}
    for p in svc.get("spec", {}).get("ports", []):
        name = p.get("name") or "service"
        out[str(name)] = int(p.get("port"))
    return out


def main() -> int:
    if len(sys.argv) != 5:
        print("usage: check_live_layer_snapshot.py <services.json> <deployments.json> <contract.json> <triage.json>", file=sys.stderr)
        return 2
    services_path, deployments_path, contract_path, triage_path = map(Path, sys.argv[1:5])
    services = json.loads(services_path.read_text(encoding="utf-8"))
    deployments = json.loads(deployments_path.read_text(encoding="utf-8"))
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    allowlist, allowlist_errors = _load_allowlist()

    svc_map = {item["metadata"]["name"]: item for item in services.get("items", [])}
    dep_map = {item["metadata"]["name"]: item for item in deployments.get("items", [])}
    required_labels = contract.get("labels", {}).get("required", [])

    diffs: list[dict[str, str | int | None]] = []
    for comp, cfg in contract.get("services", {}).items():
        svc_name = cfg["service_name"]
        svc = svc_map.get(svc_name)
        if svc is None:
            diffs.append({"diff_key": f"service.{comp}.missing", "component": comp, "kind": "service", "expected": svc_name, "actual": None})
            continue
        live_ports = _service_ports(svc)
        for key, expected in contract.get("ports", {}).get(comp, {}).items():
            if key == "container":
                continue
            actual = live_ports.get(key)
            if actual is None and key == "service":
                actual = next(iter(live_ports.values()), None)
            if actual != int(expected):
                diffs.append(
                    {
                        "diff_key": f"service.{comp}.port.{key}",
                        "component": comp,
                        "kind": "port",
                        "expected": int(expected),
                        "actual": actual,
                    }
                )

        selector = cfg.get("selector", {})
        dep_name = next(
            (name for name, item in dep_map.items() if all(item.get("spec", {}).get("selector", {}).get("matchLabels", {}).get(k) == v for k, v in selector.items())),
            None,
        )
        if dep_name:
            labels = dep_map[dep_name].get("metadata", {}).get("labels", {})
            for label in required_labels:
                if label not in labels:
                    diffs.append(
                        {
                            "diff_key": f"deployment.{dep_name}.label.{label}",
                            "component": comp,
                            "kind": "label",
                            "expected": "present",
                            "actual": "missing",
                        }
                    )

    suppressed: list[dict[str, str]] = []
    active: list[dict[str, str | int | None]] = []
    for diff in diffs:
        key = str(diff["diff_key"])
        exc = _is_allowlisted(key, allowlist)
        if exc:
            suppressed.append({"diff_key": key, "exception_id": exc})
        else:
            active.append(diff)

    triage = {
        "schema_version": 1,
        "contract_version": contract.get("contract_version"),
        "allowlist_errors": allowlist_errors,
        "active_differences": active,
        "suppressed_differences": suppressed,
    }
    triage_out = Path(triage_path)
    triage_out.parent.mkdir(parents=True, exist_ok=True)
    triage_out.write_text(json.dumps(triage, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(triage_out.as_posix())

    if allowlist_errors:
        print("live layer contract allowlist errors:", file=sys.stderr)
        for e in allowlist_errors:
            print(f"- {e}", file=sys.stderr)
        return 1
    if active:
        print("live layer contract drift detected; see triage output:", file=sys.stderr)
        for diff in active:
            print(f"- {diff['diff_key']}: expected={diff['expected']} actual={diff['actual']}", file=sys.stderr)
        return 1
    print("live layer contract validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
