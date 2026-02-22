#!/usr/bin/env python3
# Purpose: enforce reproducibility pin contracts and emit drift report.
# Inputs: configs/ops/pins.json and ops manifests/scripts.
# Outputs: non-zero on violations; writes artifacts/evidence/pins/<run_id>/pin-drift-report.json.
from __future__ import annotations

import datetime as dt
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
PINS_PATH = ROOT / "configs/ops/pins.json"
TOOLS_LOCK_PATH = ROOT / "configs/ops/tool-versions.json"
STACK_VERSION_MANIFEST = ROOT / "ops/stack/version-manifest.json"
RUN_ID = os.environ.get("RUN_ID") or os.environ.get("OPS_RUN_ID") or "manual"
OUT = ROOT / "artifacts" / "evidence" / "pins" / RUN_ID / "pin-drift-report.json"
PIN_RELAXATIONS_PATH = ROOT / "configs/policy/pin-relaxations.json"


def _run(cmd: list[str]) -> tuple[int, str]:
    try:
        out = subprocess.check_output(cmd, cwd=ROOT, text=True, stderr=subprocess.STDOUT)
        return 0, out
    except subprocess.CalledProcessError as exc:
        return exc.returncode, exc.output


def _validate_provenance(bucket: dict, errors: list[str], prefix: str) -> None:
    for key, value in sorted(bucket.items()):
        if not isinstance(value, dict):
            errors.append(f"{prefix}.{key} must be an object")
            continue
        for req in ("source_url", "pinned_at", "owner"):
            if not str(value.get(req, "")).strip():
                errors.append(f"{prefix}.{key} missing `{req}`")
        raw_date = str(value.get("pinned_at", ""))
        try:
            dt.date.fromisoformat(raw_date)
        except ValueError:
            errors.append(f"{prefix}.{key} has invalid pinned_at `{raw_date}`")


def _check_no_latest(errors: list[str]) -> None:
    code, out = _run(["./bin/atlasctl", "--quiet", "check", "domain", "docker"])
    if code != 0:
        errors.append("no-latest policy failed")


def _check_rendered_digests(errors: list[str], checks: dict[str, bool]) -> None:
    code, rendered = _run(
        [
            "helm",
            "template",
            "atlas",
            "ops/k8s/charts/bijux-atlas",
            "-f",
            "ops/k8s/values/local.yaml",
            "-f",
            "ops/k8s/values/perf.yaml",
        ]
    )
    if code != 0:
        errors.append("helm template failed for digest policy")
        checks["rendered_digest_policy"] = False
        return
    image_lines = [ln.strip() for ln in rendered.splitlines() if ln.strip().startswith("image:")]
    missing = [ln for ln in image_lines if "@sha256:" not in ln]
    if missing:
        errors.append("rendered manifests contain image references without digest")
        checks["rendered_digest_policy"] = False
    else:
        checks["rendered_digest_policy"] = True


def _check_helm_repo_pinning(errors: list[str], checks: dict[str, bool]) -> None:
    code, _ = _run(["bash", "ops/k8s/tests/checksuite/checks/obs/test_helm_repo_pinning.sh"])
    checks["helm_repo_pinning"] = code == 0
    if code != 0:
        errors.append("helm repo pinning check failed")


def _check_kind_drift(errors: list[str], checks: dict[str, bool]) -> None:
    code, _ = _run(["bash", "ops/k8s/tests/checksuite/checks/rollout/test_kind_version_drift.sh"])
    checks["kind_version_drift"] = code == 0
    if code != 0:
        errors.append("kind version drift check failed")


def _check_tools_ssot(pins: dict, errors: list[str], checks: dict[str, bool]) -> None:
    tools_lock = json.loads(TOOLS_LOCK_PATH.read_text(encoding="utf-8"))
    pins_tools = pins.get("tools", {})
    for tool, expected in tools_lock.items():
        pinned = str(pins_tools.get(tool, {}).get("version", ""))
        if pinned != str(expected):
            errors.append(f"tool pin drift for `{tool}`: expected {expected}, got {pinned or '<missing>'}")
    checks["tools_lock_sync"] = True


def _check_version_manifest(pins: dict, errors: list[str], checks: dict[str, bool]) -> None:
    manifest = json.loads(STACK_VERSION_MANIFEST.read_text(encoding="utf-8"))
    pins_images = pins.get("images", {})
    mapping = {
        "kind_node_image": "kind_node_image",
        "minio": "minio",
        "minio_mc": "minio_mc",
        "prometheus": "prometheus",
        "otel_collector": "otel_collector",
        "redis": "redis",
        "toxiproxy": "toxiproxy",
    }
    for manifest_key, pin_key in mapping.items():
        manifest_ref = str(manifest.get(manifest_key, ""))
        pin_ref = str(pins_images.get(pin_key, {}).get("ref", ""))
        if manifest_ref != pin_ref:
            errors.append(f"version-manifest drift `{manifest_key}`: {manifest_ref} != {pin_ref}")
    checks["version_manifest_sync"] = True


def _check_dataset_lock_truth(pins: dict, errors: list[str], checks: dict[str, bool]) -> None:
    lock_rel = str(pins.get("datasets", {}).get("lockfile", ""))
    if lock_rel != "ops/datasets/manifest.lock":
        errors.append("dataset lockfile must be ops/datasets/manifest.lock")
    manifest = json.loads((ROOT / "ops/datasets/manifest.json").read_text(encoding="utf-8"))
    for ds in manifest.get("datasets", []):
        if "checksums" in ds:
            errors.append("ops/datasets/manifest.json must not include checksums; keep checksums in manifest.lock")
    checks["dataset_lock_truth"] = True


def _check_pin_bypass_policy(errors: list[str], checks: dict[str, bool]) -> None:
    if os.environ.get("ALLOW_PIN_BYPASS", "0") != "1":
        checks["pin_bypass_policy"] = True
        return
    relax_id = os.environ.get("PIN_RELAXATION_ID", "").strip()
    if not relax_id:
        errors.append("ALLOW_PIN_BYPASS=1 requires PIN_RELAXATION_ID")
        checks["pin_bypass_policy"] = False
        return
    payload = json.loads(PIN_RELAXATIONS_PATH.read_text(encoding="utf-8"))
    today = dt.date.today()
    for entry in payload.get("exceptions", []):
        if str(entry.get("id", "")).strip() != relax_id:
            continue
        expiry = str(entry.get("expiry", "")).strip()
        try:
            expiry_date = dt.date.fromisoformat(expiry)
        except ValueError:
            errors.append(f"PIN_RELAXATION_ID `{relax_id}` has invalid expiry")
            checks["pin_bypass_policy"] = False
            return
        if expiry_date < today:
            errors.append(f"PIN_RELAXATION_ID `{relax_id}` expired on {expiry}")
            checks["pin_bypass_policy"] = False
            return
        checks["pin_bypass_policy"] = True
        return
    errors.append(f"PIN_RELAXATION_ID `{relax_id}` not found in configs/policy/pin-relaxations.json")
    checks["pin_bypass_policy"] = False


def _write_report(errors: list[str], checks: dict[str, bool]) -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": 1,
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "status": "pass" if not errors else "fail",
        "checks": checks,
        "errors": errors,
    }
    OUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    pins = json.loads(PINS_PATH.read_text(encoding="utf-8"))
    errors: list[str] = []
    checks: dict[str, bool] = {}
    if pins.get("schema_version") != 1:
        errors.append("pins.json schema_version must be 1")
    if not str(pins.get("contract_version", "")).strip():
        errors.append("pins.json contract_version is required")

    _validate_provenance(pins.get("tools", {}), errors, "tools")
    _validate_provenance(pins.get("images", {}), errors, "images")
    _validate_provenance({k: v for k, v in {"helm": pins.get("helm", {})}.items()}, errors, "meta")
    for i, entry in enumerate(pins.get("datasets", {}).get("entries", [])):
        _validate_provenance({f"entry_{i}": entry}, errors, "datasets")

    _check_tools_ssot(pins, errors, checks)
    _check_version_manifest(pins, errors, checks)
    _check_dataset_lock_truth(pins, errors, checks)
    _check_pin_bypass_policy(errors, checks)
    _check_no_latest(errors)
    _check_rendered_digests(errors, checks)
    _check_helm_repo_pinning(errors, checks)
    _check_kind_drift(errors, checks)

    _write_report(errors, checks)
    if errors:
        print("pins check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        print(OUT.as_posix())
        return 1
    print("pins check passed")
    print(OUT.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
