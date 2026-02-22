from __future__ import annotations

import datetime as dt
import re
import json
import hashlib
import subprocess
from pathlib import Path

from .check_ops_manifests_schema import run as run_ops_manifests_schema_check
from ....repo.native import (
    check_committed_generated_hygiene,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ....core.base import CheckDef

_OPS_RUN_TEMP_APPROVALS = Path("configs/policy/ops-run-temp-script-approvals.json")
_ISSUE_ID_RE = re.compile(r"^ISSUE-[A-Z0-9-]+$")


def _load_ops_run_temp_approvals(repo_root: Path) -> tuple[list[dict[str, object]], list[str]]:
    path = repo_root / _OPS_RUN_TEMP_APPROVALS
    if not path.exists():
        return [], [f"missing {_OPS_RUN_TEMP_APPROVALS.as_posix()}"]
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [], [f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: invalid json: {exc}"]
    approvals = payload.get("approvals", [])
    if not isinstance(approvals, list):
        return [], [f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: `approvals` must be a list"]
    rows: list[dict[str, object]] = []
    errors: list[str] = []
    today = dt.date.today()
    for idx, item in enumerate(approvals):
        if not isinstance(item, dict):
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}] must be an object")
            continue
        script = str(item.get("script", "")).strip()
        owner = str(item.get("owner", "")).strip()
        issue = str(item.get("issue_id", "")).strip()
        reason = str(item.get("reason", "")).strip()
        expiry = str(item.get("expiry", "")).strip()
        if not script.startswith("ops/run/"):
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].script must start with ops/run/")
        if not owner:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].owner is required")
        if not _ISSUE_ID_RE.match(issue):
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].issue_id invalid (`{issue}`)")
        if not reason:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].reason is required")
        try:
            exp_date = dt.date.fromisoformat(expiry)
        except ValueError:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].expiry must be YYYY-MM-DD")
            exp_date = None
        if exp_date is not None and exp_date < today:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}] expired on {expiry}")
        rows.append(item)
    return rows, errors


def check_ops_manifests_schema(repo_root: Path) -> tuple[int, list[str]]:
    del repo_root
    code = run_ops_manifests_schema_check()
    return code, []


def check_ops_no_direct_script_entrypoints(repo_root: Path) -> tuple[int, list[str]]:
    command_patterns = (
        re.compile(r"(?:^|\s)(?:\./)?ops/(?!run/)[A-Za-z0-9_./-]+\.(?:sh|py)\b"),
        re.compile(r"\b(?:bash|sh)\s+(?:\./)?ops/(?!run/)[A-Za-z0-9_./-]+\.(?:sh|py)\b"),
    )
    errors: list[str] = []
    scan_roots = [
        repo_root / "docs" / "development",
        repo_root / "docs" / "control-plane",
        repo_root / ".github" / "workflows",
        repo_root / "makefiles",
    ]
    for base in scan_roots:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix not in {".md", ".mk", ".yml", ".yaml"}:
                continue
            rel = path.relative_to(repo_root).as_posix()
            if rel.startswith("docs/_generated/") or rel.startswith("docs/_lint/"):
                continue
            for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
                stripped = line.strip()
                if not stripped or stripped.startswith("#"):
                    continue
                for pattern in command_patterns:
                    match = pattern.search(stripped)
                    if match is None:
                        continue
                    snippet = match.group(0).strip()
                    errors.append(f"{rel}:{lineno}: direct ops script entrypoint is forbidden (`{snippet}`)")
    return (0 if not errors else 1), sorted(set(errors))


def check_ops_scripts_are_data_only(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    manifests = repo_root / "ops" / "manifests"
    if not manifests.exists():
        return 1, ["missing ops/manifests directory"]
    for path in sorted(manifests.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(repo_root).as_posix()
        if path.suffix.lower() not in {".json", ".yaml", ".yml"}:
            errors.append(f"{rel}: manifests directory must contain json/yaml files only")
            continue
        if path.read_text(encoding="utf-8", errors="ignore").startswith("#!/"):
            errors.append(f"{rel}: data-only manifest must not be executable script")
    return (0 if not errors else 1), errors


def check_ops_lint_check_surfaces_native(repo_root: Path) -> tuple[int, list[str]]:
    cfg = repo_root / "configs" / "repo" / "surfaces.json"
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    allowed_dirs = set(payload["allowed_root_dirs"])
    allowed_files = set(payload["allowed_root_files"])
    canonical = set(payload["canonical_surfaces"])
    unknown: list[str] = []
    for path in sorted(repo_root.iterdir(), key=lambda p: p.name):
        name = path.name
        if path.is_dir() and name not in allowed_dirs and name not in canonical:
            unknown.append(f"unknown root entry: {name}")
        if path.is_file() and name not in allowed_files:
            unknown.append(f"unknown root entry: {name}")
    missing = [f"missing canonical surface: {name}" for name in sorted(canonical) if not (repo_root / name).exists()]
    errors = [*unknown, *missing]
    return (0 if not errors else 1), (["repo surface check passed"] if not errors else errors)


def check_ops_lint_layer_contract_drift_native(repo_root: Path) -> tuple[int, list[str]]:
    gen = repo_root / "ops" / "_meta" / "generate_layer_contract.py"
    contract = repo_root / "ops" / "_meta" / "layer-contract.json"
    before = contract.read_text(encoding="utf-8") if contract.exists() else ""
    proc = subprocess.run(["python3", str(gen.relative_to(repo_root))], cwd=repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        out = ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip()
        rows = [line for line in out.splitlines() if line.strip()] or ["layer contract generator failed"]
        return 1, rows
    after = contract.read_text(encoding="utf-8")
    if before != after:
        return 1, ["layer-contract drift detected: run packages/atlasctl/src/atlasctl/commands/ops/meta/generate_layer_contract.py and commit the result"]
    json.loads(after)
    return 0, ["layer contract drift check passed"]


def check_ops_load_abuse_scenarios_required_native(repo_root: Path) -> tuple[int, list[str]]:
    suites = repo_root / "ops" / "load" / "suites" / "suites.json"
    payload = json.loads(suites.read_text(encoding="utf-8"))
    by_name = {s.get("name"): s for s in payload.get("suites", []) if isinstance(s, dict)}
    errors: list[str] = []
    abuse = by_name.get("response-size-abuse")
    if not abuse:
        errors.append("missing required suite: response-size-abuse")
    else:
        run_in = set(abuse.get("run_in", []))
        if "nightly" not in run_in and "load-nightly" not in run_in:
            errors.append("response-size-abuse must run in nightly profile")
        if not abuse.get("must_pass", False):
            errors.append("response-size-abuse must have must_pass=true")
    return (0 if not errors else 1), (["abuse scenario contract passed"] if not errors else errors)


def check_ops_load_perf_baselines_native(repo_root: Path) -> tuple[int, list[str]]:
    baselines_dir = repo_root / "configs" / "ops" / "perf" / "baselines"
    schema = json.loads((repo_root / "ops" / "_schemas" / "load" / "perf-baseline.schema.json").read_text(encoding="utf-8"))
    budgets = json.loads((repo_root / "configs" / "ops" / "budgets.json").read_text(encoding="utf-8"))
    tools = json.loads((repo_root / "configs" / "ops" / "tool-versions.json").read_text(encoding="utf-8"))
    lock = repo_root / "ops" / "datasets" / "manifest.lock"
    expected_lock = hashlib.sha256(lock.read_bytes()).hexdigest()[:16]
    req = set(schema.get("required", []))
    freshness_days = int(budgets.get("k6_latency", {}).get("baseline_freshness_days", 30))
    warn_only = bool(budgets.get("k6_latency", {}).get("baseline_freshness_warn_only", True))
    today = dt.date.today()
    errors: list[str] = []
    found = sorted(baselines_dir.glob("*.json"))
    if not found:
        errors.append(f"no baselines in {baselines_dir.relative_to(repo_root)}")
    for path in found:
        data = json.loads(path.read_text(encoding="utf-8"))
        ctx = path.relative_to(repo_root).as_posix()
        for key in req:
            if key not in data:
                errors.append(f"{ctx}: missing `{key}`")
        meta = data.get("metadata", {})
        for key in ("environment", "profile", "dataset_set", "dataset_lock_hash", "k8s_profile", "replicas", "tool_versions", "captured_at"):
            if key not in meta:
                errors.append(f"{ctx}.metadata: missing `{key}`")
        try:
            captured = dt.datetime.fromisoformat(str(meta.get("captured_at", "")).replace("Z", "+00:00")).date()
            age_days = (today - captured).days
            if age_days > freshness_days and not warn_only:
                errors.append(f"{ctx}: baseline older than {freshness_days} days ({age_days}d)")
        except Exception:
            errors.append(f"{ctx}: invalid metadata.captured_at")
        if str(meta.get("dataset_lock_hash", "")) != expected_lock:
            errors.append(f"{ctx}: dataset_lock_hash mismatch (expected {expected_lock})")
        tv = meta.get("tool_versions", {})
        for key in ("k6", "kind", "kubectl", "helm"):
            expected = str(tools.get(key, ""))
            got = str(tv.get(key, ""))
            if expected and got and expected != got:
                errors.append(f"{ctx}: tool version mismatch {key}: baseline={got} expected={expected}")
    return (0 if not errors else 1), (["perf baseline contract check passed"] if not errors else errors)


def check_ops_load_pinned_queries_lock_native(repo_root: Path) -> tuple[int, list[str]]:
    src = repo_root / "ops" / "load" / "queries" / "pinned-v1.json"
    lock_path = repo_root / "ops" / "load" / "queries" / "pinned-v1.lock"
    schema_path = repo_root / "ops" / "_schemas" / "load" / "pinned-queries-lock.schema.json"
    queries = json.loads(src.read_text(encoding="utf-8"))
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    errors: list[str] = []
    if not isinstance(lock, dict):
        return 1, ["pinned query lock must be object"]
    for key in schema.get("required", []):
        if key not in lock:
            errors.append(f"pinned query lock missing required key: {key}")
    file_hash = hashlib.sha256(src.read_bytes()).hexdigest()
    if lock.get("file_sha256") != file_hash:
        errors.append("pinned query lock drift: file hash mismatch")
    expected: dict[str, str] = {}
    for group in ("cheap", "heavy"):
        for q in queries.get(group, []):
            expected[q] = hashlib.sha256(q.encode()).hexdigest()
    if lock.get("query_hashes") != expected:
        errors.append("pinned query lock drift: query hash mismatch")
    return (0 if not errors else 1), (["pinned query lock check passed"] if not errors else errors)


def check_ops_load_runbook_suite_names_native(repo_root: Path) -> tuple[int, list[str]]:
    manifest = json.loads((repo_root / "ops" / "load" / "suites" / "suites.json").read_text(encoding="utf-8"))
    runbook = (repo_root / "docs" / "operations" / "runbooks" / "load-failure-triage.md").read_text(encoding="utf-8")
    missing = [str(s.get("name")) for s in manifest.get("suites", []) if isinstance(s, dict) and s.get("name") and str(s.get("name")) not in runbook]
    if missing:
        return 1, [f"runbook missing suite name: {name}" for name in missing]
    return 0, ["runbook suite-name coverage passed"]


def check_ops_obs_endpoint_metrics_coverage_native(repo_root: Path) -> tuple[int, list[str]]:
    spec = json.loads((repo_root / "configs/openapi/v1/openapi.generated.json").read_text(encoding="utf-8"))
    coverage = json.loads((repo_root / "ops/obs/contract/endpoint-observability-contract.json").read_text(encoding="utf-8"))
    metrics = json.loads((repo_root / "ops/obs/contract/metrics-contract.json").read_text(encoding="utf-8"))
    budgets = json.loads((repo_root / "configs/ops/obs/budgets.json").read_text(encoding="utf-8"))
    known_metrics = set(metrics.get("required_metric_specs", {}).keys())
    required_by_class = budgets.get("endpoint_class_metric_requirements", {})
    endpoints = {
        (path, method)
        for path, methods in spec.get("paths", {}).items()
        if path.startswith("/v1/")
        for method in methods.keys()
    }
    covered = {(e["path"], e["method"]) for e in coverage.get("endpoints", [])}
    errors: list[str] = []
    for p, m in sorted(endpoints - covered):
        errors.append(f"missing endpoint coverage entry: {m.upper()} {p}")
    for entry in coverage.get("endpoints", []):
        klass = entry.get("class")
        if klass not in {"cheap", "medium", "heavy"}:
            errors.append(f"invalid endpoint class `{klass}` for {entry.get('method')} {entry.get('path')}")
        class_required = set(required_by_class.get(klass, []))
        endpoint_metrics = set(entry.get("required_metrics", []))
        miss = sorted(class_required - endpoint_metrics)
        if miss:
            errors.append(f"endpoint {entry.get('method')} {entry.get('path')} missing class-required metrics: {', '.join(miss)}")
        for metric in entry.get("required_metrics", []):
            if metric not in known_metrics:
                errors.append(f"unknown metric `{metric}` for endpoint {entry.get('method')} {entry.get('path')}")
    return (0 if not errors else 1), (["endpoint metric coverage check passed"] if not errors else errors)


def check_ops_obs_endpoint_trace_coverage_native(repo_root: Path) -> tuple[int, list[str]]:
    spec = json.loads((repo_root / "configs/openapi/v1/openapi.generated.json").read_text(encoding="utf-8"))
    coverage = json.loads((repo_root / "ops/obs/contract/endpoint-observability-contract.json").read_text(encoding="utf-8"))
    trace = json.loads((repo_root / "docs/contracts/TRACE_SPANS.json").read_text(encoding="utf-8"))
    budgets = json.loads((repo_root / "configs/ops/obs/budgets.json").read_text(encoding="utf-8"))
    endpoints = {
        (path, method)
        for path, methods in spec.get("paths", {}).items()
        if path.startswith("/v1/")
        for method in methods.keys()
    }
    covered = {(e["path"], e["method"]) for e in coverage.get("endpoints", [])}
    errors: list[str] = []
    for p, m in sorted(endpoints - covered):
        errors.append(f"missing endpoint trace coverage entry: {m.upper()} {p}")
    span_names = {trace.get("request_root_span", {}).get("name", "")}
    span_names.update(s.get("name", "") for s in trace.get("spans", []))
    root_attrs = set(trace.get("request_root_span", {}).get("required_attributes", []))
    span_attrs: set[str] = set()
    for span in trace.get("spans", []):
        span_attrs.update(span.get("required_attributes", []))
    known_attrs = root_attrs | span_attrs
    class_attr_requirements = budgets.get("span_attribute_requirements", {})
    for entry in coverage.get("endpoints", []):
        klass = entry.get("class")
        needed = set(class_attr_requirements.get(klass, []))
        miss = sorted(needed - known_attrs)
        if miss:
            errors.append(f"endpoint {entry.get('method')} {entry.get('path')} class `{klass}` requires unknown trace attrs: {', '.join(miss)}")
        for span in entry.get("required_trace_spans", []):
            if span not in span_names:
                errors.append(f"unknown trace span `{span}` for endpoint {entry.get('method')} {entry.get('path')}")
    return (0 if not errors else 1), (["endpoint trace coverage check passed"] if not errors else errors)


def check_ops_obs_budgets_native(repo_root: Path) -> tuple[int, list[str]]:
    budgets = json.loads((repo_root / "configs/ops/obs/budgets.json").read_text(encoding="utf-8"))
    metrics = json.loads((repo_root / "ops/obs/contract/metrics-contract.json").read_text(encoding="utf-8"))
    metric_specs = metrics.get("required_metric_specs", {})
    errors: list[str] = []
    for metric, labels in budgets.get("required_metric_labels", {}).items():
        spec = metric_specs.get(metric)
        if not isinstance(spec, dict):
            errors.append(f"obs budget references unknown metric `{metric}`")
            continue
        spec_labels = set(spec.get("required_labels", []))
        missing = sorted(set(labels) - spec_labels)
        if missing:
            errors.append(f"metric `{metric}` missing required labels from budget: {', '.join(missing)}")
    return (0 if not errors else 1), (["observability budgets check passed"] if not errors else errors)


def check_ops_obs_profile_goldens_native(repo_root: Path) -> tuple[int, list[str]]:
    payload = json.loads((repo_root / "ops/obs/contract/goldens/profiles.json").read_text(encoding="utf-8"))
    errors: list[str] = []
    profiles = payload.get("profiles", {})
    for profile in ("local", "perf", "offline"):
        if profile not in profiles:
            errors.append(f"missing golden profile: {profile}")
            continue
        spec = profiles[profile]
        for key in ("metrics_golden", "trace_golden"):
            rel = spec.get(key)
            if not isinstance(rel, str) or not rel:
                errors.append(f"profile {profile} missing {key}")
                continue
            path = repo_root / rel
            if not path.exists():
                errors.append(f"profile {profile} missing file: {rel}")
            elif path.stat().st_size == 0:
                errors.append(f"profile {profile} has empty file: {rel}")
            elif key == "trace_golden":
                try:
                    json.loads(path.read_text(encoding="utf-8"))
                except Exception as exc:
                    errors.append(f"profile {profile} invalid json in {rel}: {exc}")
    return (0 if not errors else 1), (["obs profile goldens check passed"] if not errors else errors)


def check_ops_shell_policy(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    run_dir = repo_root / "ops" / "run"
    if not run_dir.exists():
        return 0, ["ops run shell policy check passed (ops/run retired)"]
    for path in sorted(run_dir.glob("*.sh")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if '. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/lib/ops_common.sh"' not in text:
            errors.append(f"{rel}: must source packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/lib/ops_common.sh")
        if "ops_entrypoint_start " not in text:
            errors.append(f"{rel}: missing ops_entrypoint_start")
        if "ops_version_guard " not in text and path.name != "prereqs.sh":
            errors.append(f"{rel}: missing ops_version_guard")
    return (0 if not errors else 1), errors


def check_ops_effect_boundary_imports(repo_root: Path) -> tuple[int, list[str]]:
    """Ops command modules must not import CLI parser layers directly."""
    errors: list[str] = []
    ops_root = repo_root / "packages/atlasctl/src/atlasctl/commands/ops"
    for path in sorted(ops_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            stripped = line.strip()
            if stripped.startswith("#"):
                continue
            if "import atlasctl.cli" in stripped or "from atlasctl.cli" in stripped:
                errors.append(f"{rel}:{lineno}: ops command must not import atlasctl.cli layer")
            if re.search(r"from\s+atlasctl\.commands\.[a-z0-9_]+\.command\s+import", stripped):
                errors.append(f"{rel}:{lineno}: ops command must not import other command parser modules")
    return (0 if not errors else 1), errors


def check_ops_env_schema_is_used(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    parser = repo_root / "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_parser.py"
    runmod = repo_root / "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_run.py"
    ops_mk = repo_root / "makefiles/ops.mk"
    parser_text = parser.read_text(encoding="utf-8", errors="ignore") if parser.exists() else ""
    run_text = runmod.read_text(encoding="utf-8", errors="ignore") if runmod.exists() else ""
    mk_text = ops_mk.read_text(encoding="utf-8", errors="ignore") if ops_mk.exists() else ""
    if "configs/ops/env.schema.json" not in parser_text:
        errors.append("ops parser must default env schema to configs/ops/env.schema.json")
    if "ops_env_cmd" not in run_text or "validate" not in run_text or "print" not in run_text:
        errors.append("ops runtime must expose env validate/print handlers")
    if "./bin/atlasctl ops " not in mk_text:
        errors.append("makefiles/ops.mk must delegate to atlasctl ops interface")
    return (0 if not errors else 1), errors


def check_ops_pins_update_is_deterministic(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pins = repo_root / "configs/ops/pins.json"
    if not pins.exists():
        return 1, ["missing configs/ops/pins.json"]
    before = pins.read_text(encoding="utf-8")
    try:
        payload = json.loads(before)
    except json.JSONDecodeError as exc:
        return 1, [f"configs/ops/pins.json invalid json: {exc}"]
    canonical = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if before != canonical:
        errors.append("configs/ops/pins.json must be canonical sorted JSON")
    # schema stability check for required top-level keys
    required = {"schema_version", "contract_version", "tools", "images", "helm", "datasets", "policy"}
    missing = sorted(required - set(payload.keys()))
    if missing:
        errors.append(f"configs/ops/pins.json missing required keys: {', '.join(missing)}")
    return (0 if not errors else 1), errors


def check_ops_pins_no_unpinned_versions(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pins = repo_root / "configs/ops/pins.json"
    if not pins.exists():
        return 1, ["missing configs/ops/pins.json"]
    payload = json.loads(pins.read_text(encoding="utf-8"))
    tools = payload.get("tools", {})
    if not isinstance(tools, dict):
        errors.append("configs/ops/pins.json `tools` must be an object")
    else:
        for name, spec in sorted(tools.items()):
            if not isinstance(spec, dict):
                errors.append(f"configs/ops/pins.json `tools.{name}` must be an object")
                continue
            version = str(spec.get("version", "")).strip().lower()
            if not version:
                errors.append(f"configs/ops/pins.json `tools.{name}` missing version")
                continue
            if version in {"latest", "main", "master"}:
                errors.append(f"configs/ops/pins.json `tools.{name}` uses floating version `{version}`")
    images = payload.get("images", {})
    if not isinstance(images, dict):
        errors.append("configs/ops/pins.json `images` must be an object")
    else:
        for name, spec in sorted(images.items()):
            if not isinstance(spec, dict):
                errors.append(f"configs/ops/pins.json `images.{name}` must be an object")
                continue
            ref = str(spec.get("ref", "")).strip().lower()
            if not ref:
                errors.append(f"configs/ops/pins.json `images.{name}` missing ref")
                continue
            if ref.endswith(":latest") or ref in {"latest", "main", "master"}:
                errors.append(f"configs/ops/pins.json `images.{name}` uses floating ref `{ref}`")
    helm = payload.get("helm", {})
    if not isinstance(helm, dict):
        errors.append("configs/ops/pins.json `helm` must be an object")
    else:
        deps = helm.get("chart_dependencies", [])
        if deps is None:
            deps = []
        if not isinstance(deps, list):
            errors.append("configs/ops/pins.json `helm.chart_dependencies` must be a list")
        else:
            for idx, dep in enumerate(deps):
                if not isinstance(dep, dict):
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` must be an object")
                    continue
                version = str(dep.get("version", "")).strip().lower()
                if not version:
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` missing version")
                    continue
                if version in {"latest", "main", "master"}:
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` uses floating version `{version}`")
    return (0 if not errors else 1), errors


def check_ops_stack_versions_report_valid(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    src = repo_root / "configs/ops/tool-versions.json"
    out = repo_root / "ops/stack/versions.json"
    if not src.exists() or not out.exists():
        return 1, ["missing configs/ops/tool-versions.json or ops/stack/versions.json"]
    src_payload = json.loads(src.read_text(encoding="utf-8"))
    out_payload = json.loads(out.read_text(encoding="utf-8"))
    src_tools = src_payload.get("tools", {}) if isinstance(src_payload, dict) else {}
    out_tools = out_payload.get("tools", {}) if isinstance(out_payload, dict) else {}
    if src_tools != out_tools:
        errors.append("ops/stack/versions.json must match configs/ops/tool-versions.json tools map")
    return (0 if not errors else 1), errors


def check_ops_generated_not_tracked_unless_allowed(repo_root: Path) -> tuple[int, list[str]]:
    # Alias-level policy check for generated outputs.
    return check_ops_generated_tracked(repo_root)


def check_ops_scripts_count_nonincreasing(repo_root: Path) -> tuple[int, list[str]]:
    baseline_path = repo_root / "configs/policy/ops-scripts-count-baseline.json"
    if not baseline_path.exists():
        return 1, [f"missing baseline file: {baseline_path.relative_to(repo_root).as_posix()}"]
    payload = json.loads(baseline_path.read_text(encoding="utf-8"))
    baseline = int(payload.get("max_count", 0))
    patterns = ("*.sh", "*.py")
    roots = [repo_root / "ops" / "run", repo_root / "ops" / "k8s", repo_root / "ops" / "obs", repo_root / "ops" / "load", repo_root / "ops" / "datasets", repo_root / "ops" / "stack", repo_root / "ops" / "e2e"]
    count = 0
    for base in roots:
        if not base.exists():
            continue
        for pat in patterns:
            count += len([p for p in base.rglob(pat) if p.is_file()])
    if baseline <= 0:
        return 1, ["configs/policy/ops-scripts-count-baseline.json must define max_count > 0"]
    if count > baseline:
        return 1, [f"ops script migration gate regressed: count={count} baseline={baseline}"]
    return 0, []


def check_ops_run_only_allowlisted_scripts(repo_root: Path) -> tuple[int, list[str]]:
    run_dir = repo_root / "ops" / "run"
    if not run_dir.exists():
        return 0, []
    allowlisted = {
        "CONTRACT.md",
        "INDEX.md",
        "OWNER.md",
        "README.md",
    }
    approvals, approval_errors = _load_ops_run_temp_approvals(repo_root)
    approved_scripts = {
        str(item.get("script", "")).removeprefix("ops/run/").strip()
        for item in approvals
        if isinstance(item, dict)
    }
    errors: list[str] = []
    errors.extend(approval_errors)
    for path in sorted(run_dir.rglob("*")):
        if path.is_dir():
            continue
        rel = path.relative_to(run_dir).as_posix()
        if rel in allowlisted:
            continue
        if rel in approved_scripts:
            continue
        if path.suffix not in {".sh", ".py"}:
            continue
        errors.append(f"ops/run contains non-allowlisted script: ops/run/{rel}")
    return (0 if not errors else 1), errors


def check_ops_run_non_executable_unless_allowlisted(repo_root: Path) -> tuple[int, list[str]]:
    run_dir = repo_root / "ops" / "run"
    if not run_dir.exists():
        return 1, ["missing ops/run directory"]
    approvals, approval_errors = _load_ops_run_temp_approvals(repo_root)
    executable_allowlist = {
        str(item.get("script", "")).removeprefix("ops/run/").strip()
        for item in approvals
        if isinstance(item, dict)
    }
    errors: list[str] = []
    errors.extend(approval_errors)
    for path in sorted(run_dir.rglob("*")):
        if not path.is_file() or path.suffix not in {".sh", ".py"}:
            continue
        rel = path.relative_to(run_dir).as_posix()
        mode = path.stat().st_mode
        is_exec = bool(mode & 0o111)
        if is_exec and rel not in executable_allowlist:
            errors.append(f"ops/run/{rel}: executable bit forbidden (allowlist only)")
    return (0 if not errors else 1), errors


def check_ops_run_temp_script_approvals(repo_root: Path) -> tuple[int, list[str]]:
    approvals, errors = _load_ops_run_temp_approvals(repo_root)
    for item in approvals:
        script = str(item.get("script", "")).strip()
        if not script:
            continue
        if not (repo_root / script).exists():
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approved script missing on disk (`{script}`)")
    return (0 if not errors else 1), sorted(errors)


def check_ops_no_new_run_scripts_without_approval_and_expiry(repo_root: Path) -> tuple[int, list[str]]:
    errs: list[str] = []
    for fn in (check_ops_run_only_allowlisted_scripts, check_ops_run_temp_script_approvals):
        code, rows = fn(repo_root)
        if code != 0:
            errs.extend(rows)
    return (0 if not errs else 1), sorted(set(errs))




def _run_ops_script_check(repo_root: Path, script_rel: str) -> tuple[int, list[str]]:
    path = repo_root / script_rel
    if not path.exists():
        return 1, [f"missing {script_rel}"]
    if path.suffix == ".py":
        cmd = ["python3", script_rel]
    elif path.suffix == ".sh":
        cmd = ["bash", script_rel]
    else:
        return 1, [f"unsupported check script type: {script_rel}"]
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    output = ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip()
    rows = [line for line in output.splitlines() if line.strip()]
    return proc.returncode, rows


def _make_ops_script_check(script_rel: str):
    def _check(repo_root: Path) -> tuple[int, list[str]]:
        return _run_ops_script_check(repo_root, script_rel)
    _check.__name__ = "check_" + re.sub(r"[^a-z0-9]+", "_", script_rel.lower()).strip("_")
    return _check

def check_ops_obs_drift_goldens(repo_root: Path) -> tuple[int, list[str]]:
    scripts = [
        ["python3", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_profile_goldens.py"],
        ["python3", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_golden.py"],
        ["python3", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_golden.py"],
    ]
    errors: list[str] = []
    for cmd in scripts:
        proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
        if proc.returncode != 0:
            msg = ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip().splitlines()
            first = msg[0] if msg else "golden drift check failed"
            errors.append(f"{' '.join(cmd)} => {first}")
    return (0 if not errors else 1), errors


def check_ops_chart_version_contract(repo_root: Path) -> tuple[int, list[str]]:
    chart = repo_root / "ops/k8s/charts/bijux-atlas/Chart.yaml"
    if not chart.exists():
        return 1, ["missing ops/k8s/charts/bijux-atlas/Chart.yaml"]
    text = chart.read_text(encoding="utf-8", errors="ignore")
    version = ""
    app_version = ""
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith("version:"):
            version = line.split(":", 1)[1].strip().strip('"').strip("'")
        if line.startswith("appVersion:"):
            app_version = line.split(":", 1)[1].strip().strip('"').strip("'")
    errors: list[str] = []
    semver_re = re.compile(r"^\d+\.\d+\.\d+(?:[-+][A-Za-z0-9.-]+)?$")
    if not version:
        errors.append("Chart.yaml missing version")
    elif not semver_re.match(version):
        errors.append(f"Chart.yaml version must be semver, got `{version}`")
    if not app_version:
        errors.append("Chart.yaml missing appVersion")
    elif not semver_re.match(app_version):
        errors.append(f"Chart.yaml appVersion must be semver, got `{app_version}`")
    if version and app_version and version != app_version:
        errors.append(f"Chart.yaml version/appVersion mismatch (`{version}` != `{app_version}`)")
    return (0 if not errors else 1), errors


def check_ops_clean_allowed_roots_only(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_commands.py"
    if not path.exists():
        return 1, ["missing ops runtime commands module"]
    text = path.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    if 'ctx.repo_root / "ops" / "_generated"' not in text:
        errors.append("ops clean must target only ops/_generated root")
    if "shutil.rmtree(child)" not in text:
        errors.append("ops clean implementation changed; expected child-only deletion under ops/_generated")
    forbidden_markers = ['ctx.repo_root / "artifacts"', 'ctx.repo_root / "ops" / "_generated_committed"', "repo_root / \"/\""]
    for marker in forbidden_markers:
        if marker in text:
            errors.append(f"ops clean must not target non-allowed root marker `{marker}`")
    return (0 if not errors else 1), errors


_OPS_SCRIPT_CHECKS: tuple[tuple[str, str, str, callable], ...] = (
    ("ops.script.ops_lint_lane_budget_check_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/lint/policy/lane_budget_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/lint/policy/lane_budget_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/lint/policy/lane_budget_check.py")),
    ("ops.script.ops_lint_ops_smoke_budget_check_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py")),
    ("ops.script.ops_lint_policy_lane_budget_check_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/lint/policy/lane_budget_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/lint/policy/lane_budget_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/lint/policy/lane_budget_check.py")),
    ("ops.script.ops_lint_policy_ops_smoke_budget_check_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py")),
    ("ops.script.ops_datasets_scripts_py_cache_budget_check_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_budget_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_budget_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_budget_check.py")),
    ("ops.script.ops_datasets_scripts_py_cache_threshold_check_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_threshold_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_threshold_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_threshold_check.py")),
    ("ops.script.ops_load_scripts_check_baseline_update_policy_sh", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_baseline_update_policy.py`", "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_baseline_update_policy.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_baseline_update_policy.py")),
    ("ops.script.ops_load_scripts_check_prereqs_sh", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_prereqs.py`", "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_prereqs.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_prereqs.py")),
    ("ops.script.ops_load_scripts_check_regression_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_regression.py`", "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_regression.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_regression.py")),
    ("ops.script.ops_load_scripts_regression_check_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/load/baseline/regression_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/load/baseline/regression_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/load/baseline/regression_check.py")),
    ("ops.script.ops_obs_scripts_check_metric_cardinality_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/observability/check_metric_cardinality.py`", "packages/atlasctl/src/atlasctl/commands/ops/observability/check_metric_cardinality.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/observability/check_metric_cardinality.py")),
    ("ops.script.ops_obs_scripts_check_pack_upgrade_sh", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/observability/check_pack_upgrade.py`", "packages/atlasctl/src/atlasctl/commands/ops/observability/check_pack_upgrade.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/observability/check_pack_upgrade.py")),
    ("ops.script.ops_obs_scripts_check_pack_versions_sh", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/observability/check_pack_versions.py`", "packages/atlasctl/src/atlasctl/commands/ops/observability/check_pack_versions.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/observability/check_pack_versions.py")),
    ("ops.script.ops_obs_scripts_contracts_check_alerts_contract_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_alerts_contract.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_alerts_contract.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_alerts_contract.py")),
    ("ops.script.ops_obs_scripts_contracts_check_dashboard_contract_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_dashboard_contract.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_dashboard_contract.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_dashboard_contract.py")),
    ("ops.script.ops_obs_scripts_contracts_check_dashboard_metric_compat_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_dashboard_metric_compat.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_dashboard_metric_compat.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_dashboard_metric_compat.py")),
    ("ops.script.ops_obs_scripts_contracts_check_metrics_contract_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_contract.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_contract.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_contract.py")),
    ("ops.script.ops_obs_scripts_contracts_check_metrics_coverage_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_coverage.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_coverage.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_coverage.py")),
    ("ops.script.ops_obs_scripts_contracts_check_metrics_drift_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_drift.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_drift.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_drift.py")),
    ("ops.script.ops_obs_scripts_contracts_check_metrics_golden_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_golden.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_golden.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_metrics_golden.py")),
    ("ops.script.ops_obs_scripts_contracts_check_observability_lag_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_observability_lag.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_observability_lag.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_observability_lag.py")),
    ("ops.script.ops_obs_scripts_contracts_check_overload_behavior_contract_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_overload_behavior_contract.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_overload_behavior_contract.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_overload_behavior_contract.py")),
    ("ops.script.ops_obs_scripts_contracts_check_runtime_metrics_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_runtime_metrics.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_runtime_metrics.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_runtime_metrics.py")),
    ("ops.script.ops_obs_scripts_contracts_check_trace_coverage_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_coverage.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_coverage.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_coverage.py")),
    ("ops.script.ops_obs_scripts_contracts_check_trace_golden_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_golden.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_golden.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_golden.py")),
    ("ops.script.ops_obs_scripts_contracts_check_tracing_contract_py", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_tracing_contract.py`", "packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_tracing_contract.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_tracing_contract.py")),
    ("ops.script.ops_stack_scripts_idempotency_check_sh", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/stack/idempotency_check.py`", "packages/atlasctl/src/atlasctl/commands/ops/stack/idempotency_check.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/commands/ops/stack/idempotency_check.py")),
    ("ops.script.ops_stack_tests_check_live_layer_snapshot_py", "run ops script check `ops/stack/tests/check_live_layer_snapshot.py`", "ops/stack/tests/check_live_layer_snapshot.py", _make_ops_script_check("ops/stack/tests/check_live_layer_snapshot.py")),
    ("ops.script.ops_vendor_layout_checks_check_artifacts_allowlist_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_artifacts_allowlist.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_artifacts_allowlist.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_artifacts_allowlist.py")),
    ("ops.script.ops_vendor_layout_checks_check_artifacts_policy_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_artifacts_policy.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_artifacts_policy.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_artifacts_policy.py")),
    ("ops.script.ops_vendor_layout_checks_check_kind_cluster_contract_drift_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_kind_cluster_contract_drift.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_kind_cluster_contract_drift.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_kind_cluster_contract_drift.py")),
    ("ops.script.ops_vendor_layout_checks_check_no_root_dumping_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/governance/check_no_root_dumping.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/governance/check_no_root_dumping.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/governance/check_no_root_dumping.py")),
    ("ops.script.ops_vendor_layout_checks_check_ops_canonical_shims_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_canonical_shims.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_canonical_shims.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_canonical_shims.py")),
    ("ops.script.ops_vendor_layout_checks_check_ops_lib_canonical_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_ops_lib_canonical.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_ops_lib_canonical.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_ops_lib_canonical.py")),
    ("ops.script.ops_vendor_layout_checks_check_ops_script_targets_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_script_targets.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_script_targets.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_script_targets.py")),
    ("ops.script.ops_vendor_layout_checks_check_ops_stack_order_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_stack_order.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_stack_order.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_stack_order.py")),
    ("ops.script.ops_vendor_layout_checks_check_ops_workspace_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_workspace.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_workspace.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_workspace.py")),
    ("ops.script.ops_vendor_layout_checks_check_repo_hygiene_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/governance/check_repo_hygiene.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/governance/check_repo_hygiene.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/governance/check_repo_hygiene.py")),
    ("ops.script.ops_vendor_layout_checks_check_stack_manifest_consolidation_sh", "run ops script check `packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_stack_manifest_consolidation.py`", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_stack_manifest_consolidation.py", _make_ops_script_check("packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_stack_manifest_consolidation.py")),
)


def check_ops_all_check_scripts_registered(repo_root: Path) -> tuple[int, list[str]]:
    registered = {script_rel for (_, _, script_rel, _) in _OPS_SCRIPT_CHECKS}
    discovered: list[str] = []
    for path in sorted((repo_root / "ops").rglob("*")):
        if not path.is_file() or path.suffix not in {".py", ".sh"}:
            continue
        name = path.name
        if not re.search(r"(^|[_-])check([._-]|$)", name) and not name.startswith("check_"):
            continue
        discovered.append(path.relative_to(repo_root).as_posix())
    missing = sorted(set(discovered) - registered)
    return (0 if not missing else 1), [f"ops check script not registered in atlasctl checks: {p}" for p in missing]


CHECKS: tuple[CheckDef, ...] = (
    CheckDef("ops.no_tracked_generated", "ops", "forbid tracked files in generated ops dirs", 800, check_ops_generated_tracked, fix_hint="Untrack generated ops files."),
    CheckDef("ops.generated_not_tracked_unless_allowed", "ops", "forbid tracked generated ops outputs unless explicitly allowlisted", 800, check_ops_generated_not_tracked_unless_allowed, fix_hint="Keep generated outputs untracked or move to committed generated roots."),
    CheckDef("ops.no_tracked_timestamps", "ops", "forbid tracked timestamped paths", 1000, check_tracked_timestamp_paths, fix_hint="Remove timestamped tracked paths."),
    CheckDef("ops.committed_generated_hygiene", "ops", "validate deterministic committed generated assets", 1000, check_committed_generated_hygiene, fix_hint="Regenerate committed outputs deterministically."),
    CheckDef("ops.manifests_schema", "ops", "validate ops manifests against atlas.ops.manifest.v1 schema", 1000, check_ops_manifests_schema, fix_hint="Fix ops/manifests/*.json|*.yaml to satisfy atlas.ops.manifest.v1."),
    CheckDef("ops.no_direct_script_entrypoints", "ops", "forbid direct ops script entrypoints in docs/workflows/makefiles", 1000, check_ops_no_direct_script_entrypoints, fix_hint="Use ./bin/atlasctl ops ... or make wrappers, not ops/**/*.sh paths."),
    CheckDef("ops.scripts_are_data_only", "ops", "enforce ops/manifests data-only file policy", 1000, check_ops_scripts_are_data_only, fix_hint="Keep ops/manifests to json/yaml data only."),
    CheckDef("ops.shell_policy", "ops", "enforce shell runtime guard requirements for ops/run wrappers", 1000, check_ops_shell_policy, fix_hint="Source common.sh and call ops_entrypoint_start + ops_version_guard in ops/run/*.sh."),
    CheckDef("ops.effect_boundary_imports", "ops", "forbid ops command layer imports from CLI parser modules", 1000, check_ops_effect_boundary_imports, fix_hint="Keep ops commands behind runtime/effects layers and avoid importing cli parser modules."),
    CheckDef("ops.env_schema_is_used", "ops", "ensure ops env schema is the single interface for env validate/print", 1000, check_ops_env_schema_is_used, fix_hint="Use ops env validate/print with configs/ops/env.schema.json as default."),
    CheckDef("ops.pins_update_is_deterministic", "ops", "require deterministic canonical ops pins payload", 1000, check_ops_pins_update_is_deterministic, fix_hint="Keep configs/ops/pins.json canonical and stable."),
    CheckDef("ops.pins_no_unpinned_versions", "ops", "forbid floating/unpinned versions in ops pins", 1000, check_ops_pins_no_unpinned_versions, fix_hint="Pin tools/images/helm versions and image digests."),
    CheckDef("ops.stack_versions_report_valid", "ops", "require stack versions report to match tool versions SSOT", 1000, check_ops_stack_versions_report_valid, fix_hint="Regenerate ops/stack/versions.json from configs/ops/tool-versions.json."),
    CheckDef("ops.scripts_count_nonincreasing", "ops", "enforce migration gate: ops scripts count must not increase", 1000, check_ops_scripts_count_nonincreasing, fix_hint="Reduce ops scripts count or intentionally update baseline in one reviewed change."),
    CheckDef("ops.run_only_allowlisted_scripts", "ops", "ops/run contains no scripts except explicitly allowed fixtures", 1000, check_ops_run_only_allowlisted_scripts, fix_hint="Migrate behavior into atlasctl and delete ops/run scripts or extend transitional allowlist intentionally."),
    CheckDef("ops.run_non_executable_unless_allowlisted", "ops", "ops/run scripts must be non-executable except allowlisted", 1000, check_ops_run_non_executable_unless_allowlisted, fix_hint="Remove executable bits from ops/run scripts after migration, or update allowlist intentionally."),
    CheckDef("ops.run_temp_script_approvals", "ops", "temporary ops/run migration script approvals must be valid and unexpired", 1000, check_ops_run_temp_script_approvals, fix_hint="Keep configs/policy/ops-run-temp-script-approvals.json entries valid, justified, and unexpired."),
    CheckDef("ops.no_new_run_scripts_without_approval_and_expiry", "ops", "forbid new ops/run scripts without valid temporary approval and expiry", 1000, check_ops_no_new_run_scripts_without_approval_and_expiry, fix_hint="Keep ops/run scripts allowlisted only via valid non-expired temp approvals, or migrate/delete them."),
    CheckDef("ops.obs_drift_goldens", "ops", "fail on observability golden drift (generated vs golden)", 1000, check_ops_obs_drift_goldens, fix_hint="Regenerate/fix observability goldens and contracts."),
    CheckDef("ops.chart_version_contract", "ops", "enforce chart/app version contract for bijux-atlas chart", 1000, check_ops_chart_version_contract, fix_hint="Keep Chart.yaml version and appVersion present, semver, and aligned."),
    CheckDef("ops.clean_allowed_roots_only", "ops", "ensure ops clean only targets allowed generated roots", 1000, check_ops_clean_allowed_roots_only, fix_hint="Restrict ops clean to ops/_generated child deletion only."),
    CheckDef("ops.all_check_scripts_registered", "ops", "require all ops check scripts to be registered in atlasctl checks", 1000, check_ops_all_check_scripts_registered, fix_hint="Register new ops check scripts in atlasctl checks (or migrate to atlasctl-native implementation)."),
    CheckDef("ops.script.ops_lint_check_surfaces_py", "ops", "run ops script check `ops/_lint/check-surfaces.py`", 1000, check_ops_lint_check_surfaces_native, fix_hint="Fix repo surface violations in configs/repo/surfaces.json or root entries."),
    CheckDef("ops.script.ops_lint_check_layer_contract_drift_py", "ops", "run ops script check `ops/_lint/check_layer_contract_drift.py`", 1000, check_ops_lint_layer_contract_drift_native, fix_hint="Regenerate ops/_meta/layer-contract.json and commit deterministic output."),
    CheckDef("ops.script.ops_lint_layout_check_layer_contract_drift_py", "ops", "run ops script check `ops/_lint/layout/check_layer_contract_drift.py`", 1000, check_ops_lint_layer_contract_drift_native, fix_hint="Regenerate ops/_meta/layer-contract.json and commit deterministic output."),
    CheckDef("ops.script.ops_load_scripts_check_abuse_scenarios_required_py", "ops", "run ops script check `ops/load/scripts/check_abuse_scenarios_required.py`", 1000, check_ops_load_abuse_scenarios_required_native, fix_hint="Keep required load abuse suites in ops/load/suites/suites.json with nightly+must_pass contracts."),
    CheckDef("ops.script.ops_load_scripts_check_perf_baselines_py", "ops", "run ops script check `ops/load/scripts/check_perf_baselines.py`", 1000, check_ops_load_perf_baselines_native, fix_hint="Fix perf baseline metadata, freshness, dataset lock hash, and tool-version alignment."),
    CheckDef("ops.script.ops_load_scripts_check_pinned_queries_lock_py", "ops", "run ops script check `ops/load/scripts/check_pinned_queries_lock.py`", 1000, check_ops_load_pinned_queries_lock_native, fix_hint="Regenerate pinned query lock so file/query hashes match SSOT."),
    CheckDef("ops.script.ops_load_scripts_check_runbook_suite_names_py", "ops", "run ops script check `ops/load/scripts/check_runbook_suite_names.py`", 1000, check_ops_load_runbook_suite_names_native, fix_hint="Update load runbook to include all suite names from ops/load/suites/suites.json."),
    CheckDef("ops.script.ops_obs_scripts_contracts_check_endpoint_metrics_coverage_py", "ops", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_endpoint_metrics_coverage.py`", 1000, check_ops_obs_endpoint_metrics_coverage_native, fix_hint="Fix endpoint observability metric coverage contract entries and metric references."),
    CheckDef("ops.script.ops_obs_scripts_contracts_check_endpoint_trace_coverage_py", "ops", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_endpoint_trace_coverage.py`", 1000, check_ops_obs_endpoint_trace_coverage_native, fix_hint="Fix endpoint trace coverage contract entries and TRACE_SPANS attribute/span references."),
    CheckDef("ops.script.ops_obs_scripts_contracts_check_obs_budgets_py", "ops", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_obs_budgets.py`", 1000, check_ops_obs_budgets_native, fix_hint="Align observability budgets required labels with metrics contract specs."),
    CheckDef("ops.script.ops_obs_scripts_contracts_check_profile_goldens_py", "ops", "run ops script check `packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_profile_goldens.py`", 1000, check_ops_obs_profile_goldens_native, fix_hint="Fix profile goldens registry and referenced golden files."),
    *(
        CheckDef(check_id, "ops", description, 1000, fn, fix_hint=f"Fix failures in `{script_rel}` or replace it with atlasctl-native check logic.")
        for (check_id, description, script_rel, fn) in _OPS_SCRIPT_CHECKS
    ),
)
