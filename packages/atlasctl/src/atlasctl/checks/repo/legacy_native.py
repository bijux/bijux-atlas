from __future__ import annotations

import json
import os
import re
import subprocess
from dataclasses import dataclass
from datetime import date, datetime, timezone
from fnmatch import fnmatch
from pathlib import Path


@dataclass(frozen=True)
class PythonMigrationException:
    kind: str
    path_glob: str
    contains: str


def _load_python_migration_exceptions(repo_root: Path) -> list[PythonMigrationException]:
    payload = json.loads((repo_root / "configs/layout/python-migration-exceptions.json").read_text(encoding="utf-8"))
    items: list[PythonMigrationException] = []
    for row in payload.get("exceptions", []):
        items.append(
            PythonMigrationException(
                kind=str(row.get("kind", "")),
                path_glob=str(row.get("path_glob", "")),
                contains=str(row.get("contains", "")),
            )
        )
    return items


def _find_python_migration_exception(repo_root: Path, kind: str, rel_path: str, line: str) -> PythonMigrationException | None:
    for entry in _load_python_migration_exceptions(repo_root):
        if entry.kind != kind:
            continue
        if not fnmatch(rel_path, entry.path_glob):
            continue
        if entry.contains and entry.contains not in line:
            continue
        return entry
    return None


def check_duplicate_script_names(repo_root: Path) -> tuple[int, list[str]]:
    seen: dict[str, list[str]] = {}
    errors: list[str] = []
    for path in sorted((repo_root / "scripts").rglob("*")):
        if not path.is_file() or path.suffix not in {".sh", ".py"}:
            continue
        canonical = path.stem.replace("_", "-")
        rel = path.relative_to(repo_root).as_posix()
        seen.setdefault(canonical, []).append(rel)

    for canonical, paths in sorted(seen.items()):
        stems = {Path(p).stem for p in paths}
        if len(stems) > 1:
            errors.append(f"{canonical}: {', '.join(sorted(paths))}")
    return (0 if not errors else 1), errors


def check_script_help(repo_root: Path) -> tuple[int, list[str]]:
    targets = [
        repo_root / "bin/atlasctl",
    ]
    errors: list[str] = []
    for p in targets:
        if not p.exists():
            errors.append(f"missing help-gated script: {p.relative_to(repo_root)}")
            continue
        proc = subprocess.run([str(p), "--help"], cwd=repo_root, text=True, capture_output=True, check=False)
        out = (proc.stdout or "") + (proc.stderr or "")
        if proc.returncode != 0:
            errors.append(f"{p.relative_to(repo_root)}: --help exited {proc.returncode}")
            continue
        low = out.lower()
        if "usage" not in low and "purpose" not in low and "contract" not in low:
            errors.append(f"{p.relative_to(repo_root)}: --help output missing usage/contract text")
    return (0 if not errors else 1), errors


def check_script_ownership(repo_root: Path) -> tuple[int, list[str]]:
    ownership_path = repo_root / "configs/meta/ownership.json"
    payload = json.loads(ownership_path.read_text(encoding="utf-8"))
    paths = set(payload.get("paths", {}).keys())
    errors: list[str] = []
    for p in sorted((repo_root / "scripts").rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(repo_root).as_posix()
        if rel.startswith("scripts/__pycache__"):
            continue
        matched = any(rel == path or rel.startswith(path + "/") for path in paths)
        if not matched:
            errors.append(rel)
    return (0 if not errors else 1), errors


def check_no_xtask_refs(repo_root: Path) -> tuple[int, list[str]]:
    include_roots = [
        repo_root / ".github",
        repo_root / "makefiles",
        repo_root / "configs",
        repo_root / "docs",
        repo_root / "scripts",
        repo_root / "packages",
        repo_root / "Cargo.toml",
    ]
    allowed_substrings = [
        "ADR",
        "adr",
    ]
    errors: list[str] = []
    ignore_paths = {
        "makefiles/ci.mk",
        "docs/development/task-runner-removal-map.md",
        "packages/atlasctl/src/atlasctl/check/native.py",
        "packages/atlasctl/src/atlasctl/checks/runner.py",
        "packages/atlasctl/src/atlasctl/check/command.py",
        "packages/atlasctl/src/atlasctl/checks/repo/__init__.py",
        "packages/atlasctl/tests/test_check_native.py",
    }
    for root in include_roots:
        paths: list[Path]
        if isinstance(root, Path) and root.is_file():
            paths = [root]
        elif isinstance(root, Path) and root.exists():
            paths = [p for p in sorted(root.rglob("*")) if p.is_file()]
        else:
            paths = []
        for p in paths:
            rel = p.relative_to(repo_root).as_posix()
            if rel in ignore_paths:
                continue
            if p.suffix not in {".md", ".mk", ".toml", ".yml", ".yaml", ".json", ".py", ".sh", ""}:
                continue
            text = p.read_text(encoding="utf-8", errors="ignore")
            if "xtask" not in text:
                continue
            if any(tok in rel for tok in ("adr", "ADR")):
                continue
            if any(tok in text for tok in allowed_substrings) and ("history" in text.lower()):
                continue
            errors.append(rel)
    return (0 if not errors else 1), sorted(set(errors))


def check_make_help(repo_root: Path) -> tuple[int, list[str]]:
    cmd = ["make", "-s", "help"]
    p1 = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    p2 = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    if p1.returncode != 0 or p2.returncode != 0:
        return 1, ["`make -s help` failed while validating help output"]
    if p1.stdout != p2.stdout:
        return 1, ["`make -s help` output is non-deterministic across two runs"]
    return 0, []


def check_make_forbidden_paths(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    makefiles = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    forbidden = ("xtask/", "tools/")
    for mk in makefiles:
        text = mk.read_text(encoding="utf-8", errors="ignore").splitlines()
        for idx, line in enumerate(text, start=1):
            if not line.startswith("\t"):
                continue
            for token in forbidden:
                if token in line:
                    errors.append(f"{mk.relative_to(repo_root)}:{idx}: forbidden `{token}` in make recipe")
    return (0 if not errors else 1), errors


def check_make_no_direct_python_script_invocations(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    makefiles = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    direct_py_path_re = re.compile(r"\bpython3?\s+([^\s`]+\.py)\b")
    allowed_module_re = re.compile(r"\bpython3?\s+-m\s+atlasctl(?:\b|$)")
    for mk in makefiles:
        lines = mk.read_text(encoding="utf-8", errors="ignore").splitlines()
        for idx, line in enumerate(lines, start=1):
            if not line.startswith("\t"):
                continue
            if not direct_py_path_re.search(line):
                continue
            if allowed_module_re.search(line):
                continue
            rel = mk.relative_to(repo_root).as_posix()
            if _find_python_migration_exception(repo_root, "makefiles_direct_python", rel, line) is not None:
                continue
            errors.append(f"{rel}:{idx}: direct `python path/to/script.py` invocation is forbidden in Makefiles")
    return (0 if not errors else 1), errors


def _git_ls_files(repo_root: Path, pathspecs: list[str]) -> list[str]:
    cmd = ["git", "ls-files", "--", *pathspecs]
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def _looks_like_timestamp_segment(segment: str) -> bool:
    patterns = (
        r"^\d{8}$",
        r"^\d{8}-\d{6}$",
        r"^\d{4}-\d{2}-\d{2}$",
        r"^\d{4}-\d{2}-\d{2}[T_]\d{2}[:\-]?\d{2}[:\-]?\d{2}Z?$",
    )
    return any(re.match(pat, segment) for pat in patterns)


def check_ops_generated_tracked(repo_root: Path) -> tuple[int, list[str]]:
    tracked = _git_ls_files(repo_root, ["ops/_generated"])
    errors = [f"tracked runtime artifact: {path}" for path in tracked]
    return (0 if not errors else 1), errors


def check_tracked_timestamp_paths(repo_root: Path) -> tuple[int, list[str]]:
    tracked = _git_ls_files(repo_root, ["."])
    errors: list[str] = []
    for rel in tracked:
        segments = Path(rel).parts
        if any(_looks_like_timestamp_segment(seg) for seg in segments):
            errors.append(f"tracked path contains timestamp-like segment: {rel}")
    return (0 if not errors else 1), errors


def check_committed_generated_hygiene(repo_root: Path) -> tuple[int, list[str]]:
    tracked = _git_ls_files(
        repo_root,
        ["docs/_generated", "ops/_generated_committed", "ops/_generated.example"],
    )
    forbidden_suffixes = (".log", ".stderr", ".stdout", ".tmp")
    errors: list[str] = []
    for rel in tracked:
        path = Path(rel)
        segments = path.parts
        if any(_looks_like_timestamp_segment(seg) for seg in segments):
            errors.append(f"timestamp-like path in committed generated area: {rel}")
        if any(rel.endswith(sfx) for sfx in forbidden_suffixes):
            errors.append(f"runtime/log artifact in committed generated area: {rel}")
    return (0 if not errors else 1), errors


def _load_make_command_allowlist(repo_root: Path) -> list[str]:
    allowlist = repo_root / "configs/layout/make-command-allowlist.txt"
    if not allowlist.exists():
        return []
    return [
        ln.strip()
        for ln in allowlist.read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.lstrip().startswith("#")
    ]


def _first_recipe_token(cmd: str) -> str:
    line = cmd.strip()
    while True:
        match = re.match(r"^[A-Za-z_][A-Za-z0-9_]*=(?:\"[^\"]*\"|'[^']*'|[^\s]+)\s+", line)
        if not match:
            break
        line = line[match.end() :].lstrip()
    if not line:
        return ""
    return line.split()[0]


def check_make_command_allowlist(repo_root: Path) -> tuple[int, list[str]]:
    allow = _load_make_command_allowlist(repo_root)
    if not allow:
        return 1, ["missing allowlist: configs/layout/make-command-allowlist.txt"]
    makefiles = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    skip_prefixes = ("if ", "for ", "while ", "case ", "{ ", "(", "then", "else", "fi", "do", "done")
    skip_tokens = {"\\", "-u", "-n", "-c", "-", "exit", "trap", "done;", "then", "fi;", "do"}
    violations: list[str] = []
    for mk in makefiles:
        continued = False
        phony_block = False
        for idx, raw in enumerate(mk.read_text(encoding="utf-8").splitlines(), start=1):
            if not raw.startswith("\t"):
                continued = False
                phony_block = raw.strip().startswith(".PHONY:")
                continue
            if phony_block:
                phony_block = raw.rstrip().endswith("\\")
                continue
            if continued:
                continued = raw.rstrip().endswith("\\")
                continue
            cmd = raw.lstrip()[1:].strip() if raw.lstrip().startswith("@") else raw.strip()
            continued = raw.rstrip().endswith("\\")
            if not cmd or cmd.startswith("#") or cmd.startswith("-"):
                continue
            if "$$(" in cmd or "|" in cmd:
                continue
            if ":?" in cmd:
                continue
            if any(cmd.startswith(prefix) for prefix in skip_prefixes):
                continue
            tok = _first_recipe_token(cmd)
            if not tok:
                continue
            if tok in skip_tokens:
                continue
            if tok.startswith("./") or tok.startswith('"') or tok.startswith("'"):
                continue
            if tok.startswith("$(") or tok.startswith("$${") or tok.startswith('"$('):
                continue
            if not re.fullmatch(r"[A-Za-z0-9_.+-]+", tok):
                continue
            if any(tok == item or tok.startswith(item) for item in allow):
                continue
            violations.append(f"{mk.relative_to(repo_root)}:{idx}: disallowed recipe command `{tok}`")
    return (0 if not violations else 1), violations


def check_layout_contract(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []

    # Root shape contract.
    surfaces = json.loads((repo_root / "configs/repo/surfaces.json").read_text(encoding="utf-8"))
    allow_dirs = set(surfaces.get("allowed_root_dirs", [])) | set(surfaces.get("canonical_surfaces", []))
    allow_files = {
        ln.strip()
        for ln in (repo_root / "configs/repo/root-files-allowlist.txt").read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.strip().startswith("#")
    }
    allow_files |= set(surfaces.get("allowed_root_files", []))
    for entry in sorted(repo_root.iterdir(), key=lambda p: p.name):
        if entry.name in {".git"}:
            continue
        if entry.name in {".DS_Store"}:
            continue
        if entry.is_dir():
            if entry.name not in allow_dirs:
                errors.append(f"unexpected root directory: {entry.name}")
        elif entry.is_file() or entry.is_symlink():
            if entry.name not in allow_files:
                errors.append(f"unexpected root file/symlink: {entry.name}")

    # Workflow make-only contract.
    run_line = re.compile(r"^\s*-\s*run:\s*(.+)\s*$")
    for wf in sorted((repo_root / ".github/workflows").glob("*.yml")):
        for idx, line in enumerate(wf.read_text(encoding="utf-8").splitlines(), start=1):
            m = run_line.match(line)
            if not m:
                continue
            cmd = m.group(1).strip().strip('"')
            if cmd.startswith("|"):
                errors.append(f"{wf.relative_to(repo_root)}:{idx}: multiline run block forbidden")
                continue
            if not cmd.startswith("make "):
                errors.append(f"{wf.relative_to(repo_root)}:{idx}: workflow run must use make, found `{cmd}`")

    # Legacy target names contract.
    target_re = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)
    forbidden_legacy = re.compile(r"(^|/)legacy($|-)")
    for mk in sorted((repo_root / "makefiles").glob("*.mk")):
        text = mk.read_text(encoding="utf-8")
        for target in target_re.findall(text):
            if target.startswith("."):
                continue
            if forbidden_legacy.search(target):
                errors.append(f"{mk.relative_to(repo_root)}: forbidden legacy target `{target}`")

    # Symlink policy contract.
    symlink_cfg = json.loads((repo_root / "configs/repo/symlink-allowlist.json").read_text(encoding="utf-8"))
    allowed_root = symlink_cfg.get("root", {})
    allowed_non_root = symlink_cfg.get("non_root", {})
    for rel, target in sorted(allowed_root.items()):
        p = repo_root / rel
        if not p.is_symlink():
            errors.append(f"missing allowlisted root symlink: {rel}")
            continue
        resolved = p.resolve()
        try:
            got = resolved.relative_to(repo_root).as_posix()
        except ValueError:
            errors.append(f"root symlink points outside repo: {rel}")
            continue
        if got != target:
            errors.append(f"root symlink target drift: {rel} -> {got} (expected {target})")
    for rel, target in sorted(allowed_non_root.items()):
        p = repo_root / rel
        if not p.exists():
            continue
        if not p.is_symlink():
            errors.append(f"allowlisted non-root path exists but is not symlink: {rel}")
            continue
        resolved = p.resolve()
        try:
            got = resolved.relative_to(repo_root).as_posix()
        except ValueError:
            errors.append(f"non-root symlink points outside repo: {rel}")
            continue
        if got != target:
            errors.append(f"non-root symlink target drift: {rel} -> {got} (expected {target})")

    # Existing tracked/generated hygiene contracts.
    for fn in (check_ops_generated_tracked, check_tracked_timestamp_paths, check_committed_generated_hygiene):
        _code, errs = fn(repo_root)
        errors.extend(errs)

    return (0 if not errors else 1), errors


def check_make_scripts_references(repo_root: Path) -> tuple[int, list[str]]:
    exceptions_path = repo_root / "configs/layout/make-scripts-reference-exceptions.json"
    makefiles = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    errors: list[str] = []
    exceptions: list[dict[str, str]] = []
    if exceptions_path.exists():
        payload = json.loads(exceptions_path.read_text(encoding="utf-8"))
        today = date.today()
        for raw in payload.get("exceptions", []):
            if not isinstance(raw, dict):
                continue
            rid = str(raw.get("id", "<missing-id>"))
            expiry = str(raw.get("expires_on", ""))
            try:
                exp = date.fromisoformat(expiry)
            except ValueError:
                errors.append(f"invalid expires_on for exception {rid}: `{expiry}`")
                continue
            if exp < today:
                errors.append(f"expired exception {rid}: {expiry}")
                continue
            exceptions.append({"pattern": str(raw.get("pattern", ""))})

    violations: list[str] = []
    for mk in makefiles:
        for idx, line in enumerate(mk.read_text(encoding="utf-8").splitlines(), start=1):
            if "scripts/" not in line or not line.startswith("\t"):
                continue
            if any(ex["pattern"] and ex["pattern"] in line for ex in exceptions):
                continue
            violations.append(f"{mk.relative_to(repo_root)}:{idx}: unapproved scripts/ reference in make recipe")

    errors.extend(violations)
    return (0 if not errors else 1), errors


def check_docs_scripts_references(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    allowed = {"docs/development/task-runner-removal-map.md"}
    for p in sorted((repo_root / "docs").rglob("*")):
        if not p.is_file() or p.suffix != ".md":
            continue
        rel = p.relative_to(repo_root).as_posix()
        if rel.startswith("docs/_generated/"):
            continue
        if rel in allowed:
            continue
        for idx, line in enumerate(p.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            if "scripts/" in line:
                errors.append(f"{rel}:{idx}: docs must not reference scripts/ paths")
    return (0 if not errors else 1), errors


def check_no_executable_python_outside_packages(repo_root: Path) -> tuple[int, list[str]]:
    exceptions_cfg = repo_root / "configs/layout/python-migration-exceptions.json"
    globs: list[str] = []
    if exceptions_cfg.exists():
        payload = json.loads(exceptions_cfg.read_text(encoding="utf-8"))
        globs = [str(row.get("path_glob", "")) for row in payload.get("exceptions", [])]
    errors: list[str] = []
    for p in sorted(repo_root.rglob("*.py")):
        if not p.is_file():
            continue
        rel = p.relative_to(repo_root).as_posix()
        if rel.startswith("packages/") or "/__pycache__/" in rel:
            continue
        first = p.read_text(encoding="utf-8", errors="ignore").splitlines()[:1]
        shebang = first[0] if first else ""
        if shebang.startswith("#!/usr/bin/env python"):
            if any(fnmatch(rel, g) or rel == g for g in globs if g):
                continue
            errors.append(f"{rel}: executable python outside packages/")
    return (0 if not errors else 1), errors


def check_forbidden_top_dirs(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for name in ("xtask", "tools"):
        if (repo_root / name).exists():
            errors.append(f"forbidden top-level directory exists: {name}/")
    return (0 if not errors else 1), errors


def check_docker_layout(repo_root: Path) -> tuple[int, list[str]]:
    root_df = repo_root / "Dockerfile"
    canon = repo_root / "docker" / "images" / "runtime" / "Dockerfile"
    errors: list[str] = []
    for required in [
        repo_root / "docker" / "images",
        repo_root / "docker" / "contracts",
        repo_root / "docker" / "scripts",
    ]:
        if not required.is_dir():
            errors.append(f"missing required docker directory: {required.relative_to(repo_root)}")
    if not root_df.is_symlink():
        errors.append("root Dockerfile must be a symlink to docker/images/runtime/Dockerfile")
    else:
        if root_df.resolve() != canon.resolve():
            errors.append(f"root Dockerfile symlink target drift: expected {canon}, got {root_df.resolve()}")
    for path in repo_root.rglob("Dockerfile*"):
        rel = path.relative_to(repo_root).as_posix()
        if rel == "Dockerfile" or rel.startswith("docker/"):
            continue
        if "/artifacts/" in rel or rel.startswith("artifacts/"):
            continue
        errors.append(f"Dockerfile outside docker/ namespace forbidden: {rel}")
    return (0 if not errors else 1), errors


def check_docker_policy(repo_root: Path) -> tuple[int, list[str]]:
    dockerfile = (repo_root / "docker/images/runtime/Dockerfile").read_text(encoding="utf-8")
    product_mk = (repo_root / "makefiles/product.mk").read_text(encoding="utf-8")
    allowlist = json.loads((repo_root / "docker/contracts/base-image-allowlist.json").read_text(encoding="utf-8"))
    pinning = json.loads((repo_root / "docker/contracts/digest-pinning.json").read_text(encoding="utf-8"))
    errors: list[str] = []
    if re.search(r"^FROM\s+[^\n]*:latest\b", dockerfile, re.MULTILINE):
        errors.append("docker/images/runtime/Dockerfile uses forbidden latest tag")
    if not re.search(r"^ARG\s+RUST_VERSION=([0-9]+\.[0-9]+\.[0-9]+)$", dockerfile, re.MULTILINE):
        errors.append("docker/images/runtime/Dockerfile must pin ARG RUST_VERSION=<semver>")
    from_lines = re.findall(r"^FROM\s+([^\s]+)", dockerfile, re.MULTILINE)
    if len(from_lines) < 2:
        errors.append("docker/images/runtime/Dockerfile must define builder and runtime stages")
    else:
        builder, runtime = from_lines[0], from_lines[-1]
        if not any(builder.startswith(prefix) for prefix in allowlist.get("allowed_builder_images", [])):
            errors.append(f"builder base image not in allowlist: {builder}")
        if runtime not in set(allowlist.get("allowed_runtime_images", [])):
            errors.append(f"runtime base image not in allowlist: {runtime}")
    if pinning.get("forbid_latest", False) and ":latest" in dockerfile:
        errors.append("forbid_latest=true but :latest appears in dockerfile")
    for label in [
        "org.opencontainers.image.version",
        "org.opencontainers.image.revision",
        "org.opencontainers.image.created",
        "org.opencontainers.image.source",
        "org.opencontainers.image.ref.name",
    ]:
        if label not in dockerfile:
            errors.append(f"missing required OCI label: {label}")
    if "docker build" in product_mk:
        for token in [
            "--pull=false",
            "--build-arg RUST_VERSION",
            "--build-arg IMAGE_VERSION",
            "--build-arg VCS_REF",
            "--build-arg BUILD_DATE",
        ]:
            if token not in product_mk:
                errors.append(f"docker-build target missing reproducibility/provenance arg: {token}")
    return (0 if not errors else 1), errors


def check_no_latest_tags(repo_root: Path) -> tuple[int, list[str]]:
    policy = json.loads((repo_root / "docker/contracts/no-latest.json").read_text(encoding="utf-8"))
    if not policy.get("forbid_latest", True):
        return 0, []
    needle = re.compile(r":[Ll][Aa][Tt][Ee][Ss][Tt](\b|@)")
    scan_files = [
        repo_root / "docker/images/runtime/Dockerfile",
        *sorted((repo_root / "ops").rglob("*.yaml")),
        *sorted((repo_root / "ops").rglob("*.yml")),
        *sorted((repo_root / "ops").rglob("*.sh")),
        *sorted((repo_root / "scripts").rglob("*.sh")),
        *sorted((repo_root / "makefiles").rglob("*.mk")),
    ]
    errors: list[str] = []
    for path in scan_files:
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for idx, line in enumerate(text.splitlines(), start=1):
            if "releases/latest/download" in line:
                continue
            line_l = line.lower()
            relevant = (
                "image:" in line_l
                or line_l.strip().startswith("from ")
                or "docker run" in line_l
                or "docker pull" in line_l
                or "--image=" in line_l
                or "helm upgrade" in line_l
                or "helm install" in line_l
            )
            if relevant and needle.search(line):
                errors.append(f"{path.relative_to(repo_root)}:{idx}: {line.strip()}")
    return (0 if not errors else 1), errors


def check_docker_image_size(repo_root: Path) -> tuple[int, list[str]]:
    budget = json.loads((repo_root / "docker/contracts/image-size-budget.json").read_text(encoding="utf-8"))
    image = os.environ.get("DOCKER_IMAGE", "bijux-atlas:local")
    max_bytes = int(budget["runtime_image_max_bytes"])
    proc = subprocess.run(
        ["docker", "image", "inspect", image, "--format", "{{.Size}}"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return 0, []
    try:
        size = int(proc.stdout.strip())
    except ValueError:
        return 1, [f"docker image size check failed: invalid size output '{proc.stdout.strip()}'"]
    if size > max_bytes:
        return 1, [f"docker image size budget exceeded: {size} > {max_bytes} bytes for {image}"]
    return 0, []


def check_python_migration_exceptions_expiry(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "configs" / "layout" / "python-migration-exceptions.json"
    payload = json.loads(path.read_text(encoding="utf-8"))
    today = date.today()
    errors: list[str] = []
    for row in payload.get("exceptions", []):
        expires_on = date.fromisoformat(str(row.get("expires_on", "")))
        if expires_on < today:
            errors.append(
                f"{row.get('id')} kind={row.get('kind')} owner={row.get('owner')} "
                f"expires_on={row.get('expires_on')} issue={row.get('issue')}"
            )
    return (0 if not errors else 1), errors


def check_bin_entrypoints(repo_root: Path) -> tuple[int, list[str]]:
    scripts_bin = repo_root / "scripts" / "bin"
    if not scripts_bin.exists():
        return 0, []
    files = sorted(p for p in scripts_bin.glob("*") if p.is_file())
    if len(files) > 15:
        return 1, [f"scripts/bin cap exceeded: {len(files)} > 15"]
    return 0, []


def check_python_lock(repo_root: Path) -> tuple[int, list[str]]:
    locks = [repo_root / "packages/atlasctl/requirements.lock.txt"]
    pat = re.compile(r"^[a-zA-Z0-9_.-]+==[a-zA-Z0-9_.-]+$")
    errors: list[str] = []
    for lock in locks:
        text = lock.read_text(encoding="utf-8")
        lines = [ln.strip() for ln in text.splitlines() if ln.strip() and not ln.strip().startswith("#")]
        invalid = [ln for ln in lines if not pat.match(ln)]
        for line in invalid:
            errors.append(f"{lock.relative_to(repo_root)}: {line}")
    return (0 if not errors else 1), errors


def check_scripts_lock_sync(repo_root: Path) -> tuple[int, list[str]]:
    cfg = json.loads((repo_root / "configs/scripts/python-tooling.json").read_text(encoding="utf-8"))
    errors: list[str] = []
    if cfg.get("toolchain") != "pip-tools":
        errors.append("python tooling SSOT must declare toolchain=pip-tools")
        return 1, errors
    lockfile = repo_root / str(cfg["lockfile"])
    if not lockfile.exists():
        return 1, [f"missing lockfile: {lockfile}"]
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    match = re.search(r"\[project\.optional-dependencies\]\s*dev\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    if not match:
        return 1, ["unable to parse [project.optional-dependencies].dev from pyproject.toml"]
    expected = sorted(re.findall(r'"([^"]+)"', match.group("body")))
    lines = [ln.strip() for ln in lockfile.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.startswith("#")]
    locked = sorted(lines)
    if expected != locked:
        return 1, [f"scripts lock drift: expected={expected} locked={locked}"]
    return 0, []


def check_no_adhoc_python(repo_root: Path) -> tuple[int, list[str]]:
    allowlist = repo_root / "configs/layout/python-legacy-allowlist.txt"
    allow = {
        line.strip()
        for line in allowlist.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }
    proc = subprocess.run(["git", "ls-files", "*.py"], cwd=repo_root, check=False, text=True, capture_output=True)
    errors: list[str] = []
    for rel in sorted(ln.strip() for ln in proc.stdout.splitlines() if ln.strip()):
        if rel.startswith("packages/atlasctl/"):
            continue
        if "/tests/" in rel:
            continue
        if _find_python_migration_exception(repo_root, "python_scripts_path", rel, "") is not None:
            continue
        if rel in allow:
            continue
        errors.append(rel)
    return (0 if not errors else 1), errors


def check_no_direct_python_invocations(repo_root: Path) -> tuple[int, list[str]]:
    docs = repo_root / "docs"
    makefiles = repo_root / "makefiles"
    makefile = repo_root / "Makefile"
    direct_py_re = re.compile(r"\bpython3?\s+([^\s`]+\.py)\b")
    py_scripts_re = re.compile(r"\bpython3?\s+scripts/[^\s`]+\.py\b")
    allowed_make_re = re.compile(r"\bpython3?\s+-m\s+atlasctl(?:\b|$)")
    errors: list[str] = []

    def scan(path: Path, kind: str) -> None:
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if py_scripts_re.search(line):
                if _find_python_migration_exception(repo_root, "python_scripts_path", rel, line) is None:
                    errors.append(f"{rel}:{lineno}: direct `python scripts/*.py` invocation is forbidden")
            if kind == "docs":
                if direct_py_re.search(line):
                    if _find_python_migration_exception(repo_root, "docs_direct_python", rel, line) is None:
                        errors.append(
                            f"{rel}:{lineno}: docs must reference `atlasctl`, not direct python execution"
                        )
            if kind == "makefiles":
                if direct_py_re.search(line) and not allowed_make_re.search(line):
                    if _find_python_migration_exception(repo_root, "makefiles_direct_python", rel, line) is None:
                        errors.append(
                            f"{rel}:{lineno}: makefiles must use `atlasctl` or `python -m atlasctl...`"
                        )

    for path in docs.rglob("*.md"):
        if "docs/_generated/" in path.as_posix():
            continue
        scan(path, "docs")
    for path in makefiles.glob("*.mk"):
        scan(path, "makefiles")
    scan(makefile, "makefiles")
    return (0 if not errors else 1), errors


def check_no_direct_bash_invocations(repo_root: Path) -> tuple[int, list[str]]:
    docs = repo_root / "docs"
    makefiles = repo_root / "makefiles"
    makefile = repo_root / "Makefile"
    bash_scripts_re = re.compile(r"\bbash\s+([^\s`]*scripts/[^\s`]+)\b")
    errors: list[str] = []

    def scan(path: Path, kind: str) -> None:
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if not bash_scripts_re.search(line):
                continue
            exc_kind = "docs_direct_bash" if kind == "docs" else "makefiles_direct_bash"
            if _find_python_migration_exception(repo_root, exc_kind, rel, line) is None:
                errors.append(f"{rel}:{lineno}: direct `bash ...scripts/...` invocation is forbidden")

    for path in docs.rglob("*.md"):
        if "docs/_generated/" in path.as_posix():
            continue
        scan(path, "docs")
    for path in makefiles.glob("*.mk"):
        scan(path, "makefiles")
    scan(makefile, "makefiles")
    return (0 if not errors else 1), errors


def check_docs_no_ops_generated_run_paths(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pattern = re.compile(r"ops/_generated/atlas-ops-[A-Za-z0-9._/-]*")
    for path in sorted((repo_root / "docs").rglob("*.md")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(text.splitlines(), start=1):
            if pattern.search(line):
                errors.append(f"{rel}:{lineno}: forbidden run-scoped generated path reference")
    return (0 if not errors else 1), errors


def check_no_ops_generated_placeholder(repo_root: Path) -> tuple[int, list[str]]:
    candidates = (repo_root / "ops/_generated/.keep", repo_root / "ops/_generated/.gitkeep")
    errors = [f"forbidden placeholder present: {p.relative_to(repo_root).as_posix()}" for p in candidates if p.exists()]
    return (0 if not errors else 1), errors


def check_ops_examples_immutable(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "ops/_examples"
    expected = {"report.example.json", "report.unified.example.json"}
    errors: list[str] = []
    if not root.exists():
        return 1, ["missing examples directory: ops/_examples"]
    actual = {p.name for p in root.glob("*.json")}
    for name in sorted(expected - actual):
        errors.append(f"missing example file: ops/_examples/{name}")
    for name in sorted(actual - expected):
        errors.append(f"unexpected example file: ops/_examples/{name}")
    for name in sorted(expected & actual):
        path = root / name
        if path.stat().st_size > 32 * 1024:
            errors.append(f"example too large (>32KB): {path.relative_to(repo_root).as_posix()}")
        text = path.read_text(encoding="utf-8")
        if re.search(r"atlas-ops-\d{8}", text):
            errors.append(f"example contains run-scoped atlas-ops timestamp id: {path.relative_to(repo_root).as_posix()}")
        if "ops/_generated/" in text:
            errors.append(f"example references mutable ops/_generated path: {path.relative_to(repo_root).as_posix()}")
        try:
            json.loads(text)
        except json.JSONDecodeError as exc:
            errors.append(f"invalid JSON in example {path.relative_to(repo_root).as_posix()}: {exc}")
    return (0 if not errors else 1), errors


def check_invocation_parity(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    py_mk = repo_root / "makefiles/python.mk"
    text = py_mk.read_text(encoding="utf-8")
    if "python3 -m atlasctl.cli" not in text:
        errors.append("makefiles/python.mk must invoke atlasctl via python -m atlasctl.cli")
    docs_text = (repo_root / "docs/development/tooling/atlasctl.md").read_text(
        encoding="utf-8", errors="ignore"
    )
    if re.search(r"scripts/bin/atlasctl", docs_text):
        errors.append("docs still reference scripts/bin/atlasctl")
    if "atlasctl" not in docs_text:
        errors.append("docs/development/tooling/atlasctl.md must reference atlasctl")
    return (0 if not errors else 1), errors


def check_scripts_surface_docs_drift(repo_root: Path) -> tuple[int, list[str]]:
    doc = repo_root / "docs/development/tooling/atlasctl.md"
    cfg = repo_root / "configs/scripts/python-tooling.json"
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    commands = [str(cmd) for cmd in payload.get("commands", [])]
    text = doc.read_text(encoding="utf-8")
    missing = [f"missing `{cmd}` in {doc.relative_to(repo_root)}" for cmd in commands if f"`{cmd}`" not in text]
    return (0 if not missing else 1), missing


def check_script_errors(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in sorted((repo_root / "scripts/bin").glob("bijux-atlas-*")):
        if not path.is_file() or path.name == "bijux-atlas-dev":
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "python3 -m atlasctl.cli" in text:
            continue
        if '"error_code"' not in text and "err(" not in text:
            errors.append(f"{path.relative_to(repo_root)} must emit structured JSON error_code or delegate to atlasctl")
    return (0 if not errors else 1), errors


def check_script_write_roots(repo_root: Path) -> tuple[int, list[str]]:
    allowed = (
        "artifacts/",
        "ops/_generated/",
        "ops/_generated_committed/",
        "artifacts/evidence/",
        "docs/_generated/",
        "scripts/_generated/",
    )
    write_re = re.compile(r"\b(?:>|>>|tee\s+|mkdir\s+-p\s+|cp\s+[^\n]*\s+)([^\s\"']+)")
    errors: list[str] = []
    for path in sorted((repo_root / "scripts/bin").glob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for match in write_re.finditer(text):
            target = match.group(1)
            if target.startswith("$") or target.startswith("/") or target.startswith("."):
                continue
            if any(target.startswith(prefix) for prefix in allowed):
                continue
            errors.append(f"{rel}: {target}")
    return (0 if not errors else 1), errors


def check_script_tool_guards(repo_root: Path) -> tuple[int, list[str]]:
    tool_re = re.compile(r"\b(kubectl|helm|kind|k6)\b")
    guards = ("check_tool_versions.py", "ops_version_guard", "packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py")
    errors: list[str] = []
    for scan_dir in (repo_root / "scripts/bin", repo_root / "scripts/check", repo_root / "scripts/ci"):
        if not scan_dir.exists():
            continue
        for path in sorted(scan_dir.rglob("*.sh")):
            rel = path.relative_to(repo_root).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            if not tool_re.search(text):
                continue
            if any(g in text for g in guards):
                continue
            errors.append(rel)
    return (0 if not errors else 1), errors


def check_script_shim_expiry(repo_root: Path) -> tuple[int, list[str]]:
    cfg = repo_root / "configs/layout/script-shim-expiries.json"
    data = json.loads(cfg.read_text(encoding="utf-8"))
    shims = data.get("shims", [])
    known = {entry["path"] for entry in shims if isinstance(entry, dict) and "path" in entry}
    errors: list[str] = []
    max_active = int(data.get("max_active_shims", 9999))
    shim_paths: list[str] = []
    for base in (repo_root / "scripts/bin", repo_root / "bin"):
        if not base.exists():
            continue
        for path in sorted(base.glob("*")):
            if not path.is_file() or path.name == "atlasctl":
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            if "DEPRECATED:" not in text:
                continue
            rel = path.relative_to(repo_root).as_posix()
            shim_paths.append(rel)
            if rel not in known:
                errors.append(f"shim missing expiry metadata: {rel}")
    if len(shim_paths) > max_active:
        errors.append(f"shim budget exceeded: active={len(shim_paths)} max_active_shims={max_active}")
    today = date.today()
    for row in shims:
        rel = row.get("path", "")
        if not rel:
            errors.append("shim metadata missing path")
            continue
        if not str(row.get("replacement", "")).strip():
            errors.append(f"shim metadata missing replacement command: {rel}")
        if not str(row.get("migration_doc", "")).strip():
            errors.append(f"shim metadata missing migration_doc: {rel}")
        path = repo_root / rel
        if not path.exists():
            errors.append(f"shim metadata points to missing file: {rel}")
            continue
        exp = date.fromisoformat(str(row.get("expires_on", "")))
        if exp < today:
            errors.append(f"shim expired: {rel} expired_on={exp.isoformat()}")
    return (0 if not errors else 1), errors


def check_script_shims_minimal(repo_root: Path) -> tuple[int, list[str]]:
    cfg = repo_root / "configs/layout/script-shim-expiries.json"
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    errors: list[str] = []
    for row in payload.get("shims", []):
        if not isinstance(row, dict):
            continue
        rel = str(row.get("path", ""))
        if not rel:
            continue
        path = repo_root / rel
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
        if not lines or not lines[0].startswith("#!/usr/bin/env sh"):
            errors.append(f"{rel}: shim must use portable sh shebang")
        if "DEPRECATED:" not in text:
            errors.append(f"{rel}: missing DEPRECATED warning banner")
        if "docs/development/tooling/atlasctl.md" not in text:
            errors.append(f"{rel}: missing migration doc link")
        if "exec " not in text:
            errors.append(f"{rel}: missing exec passthrough")
        if any(tok in text for tok in ("tee ", "mktemp", "touch ", "cat > ", "printf > ", "echo > ")):
            errors.append(f"{rel}: shim must not write artifacts/files")
        if "set -x" in text or "uname" in text or "if [ \"$OSTYPE\"" in text:
            errors.append(f"{rel}: shim must be deterministic and OS-neutral")
        non_comment = [ln for ln in lines if not ln.startswith("#")]
        if len(non_comment) > 2:
            errors.append(f"{rel}: shim must stay minimal (echo + exec only)")
    return (0 if not errors else 1), errors


def check_venv_location_policy(repo_root: Path) -> tuple[int, list[str]]:
    allowed_prefixes = ("artifacts/atlasctl/",)
    proc = subprocess.run(
        ["git", "ls-files", "--others", "--cached", "--exclude-standard"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    paths = [p.strip() for p in proc.stdout.splitlines() if p.strip()]
    violations: list[str] = []
    for rel in paths:
        if ".venv" not in Path(rel).parts:
            continue
        if any(rel.startswith(prefix) for prefix in allowed_prefixes):
            continue
        violations.append(rel)
    root_venv = repo_root / ".venv"
    if root_venv.exists():
        violations.append(".venv")
    return (0 if not violations else 1), violations


def check_python_runtime_artifacts(repo_root: Path, *, fix: bool = False) -> tuple[int, list[str]]:
    allowed_prefix = (repo_root / "artifacts").resolve()

    def allowed(path: Path) -> bool:
        resolved = path.resolve()
        return resolved == allowed_prefix or allowed_prefix in resolved.parents

    violations: list[str] = []
    paths_to_remove: list[Path] = []
    forbidden_dirs = {".venv", ".ruff_cache", ".pytest_cache", ".mypy_cache", "__pycache__", ".hypothesis"}
    for path in repo_root.rglob("*"):
        if path.is_dir() and path.name in forbidden_dirs and not allowed(path):
            if ".git" in path.parts:
                continue
            violations.append(f"forbidden dir outside artifacts: {path.relative_to(repo_root)}")
            paths_to_remove.append(path)
    for path in repo_root.rglob("*.pyc"):
        if not allowed(path):
            if ".git" in path.parts:
                continue
            violations.append(f"forbidden pyc outside artifacts: {path.relative_to(repo_root)}")
            paths_to_remove.append(path)
    tracked = subprocess.run(["git", "ls-files"], cwd=repo_root, check=False, text=True, capture_output=True)
    for rel in tracked.stdout.splitlines():
        if fnmatch(rel, "*.pyc"):
            violations.append(f"tracked pyc file: {rel}")
    if violations and fix:
        for path in sorted(set(paths_to_remove), key=lambda p: len(p.parts), reverse=True):
            if path.is_dir():
                subprocess.run(["rm", "-rf", str(path)], check=False)
            elif path.is_file():
                path.unlink(missing_ok=True)
        return 0, [f"python runtime artifact policy auto-fixed ({len(paths_to_remove)} paths)"]
    return (0 if not violations else 1), violations


def check_repo_script_boundaries(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    scripts_files = [p for p in _git_ls_files(repo_root, ["scripts/**"]) if not p.endswith(".md")]
    for rel in scripts_files:
        if _find_python_migration_exception(repo_root, "scripts_dir", rel, "") is None:
            errors.append(f"scripts directory transition is closed; file must move under packages/: {rel}")

    exec_proc = subprocess.run(
        ["git", "ls-files", "--stage", "*.py"],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    for line in exec_proc.stdout.splitlines():
        if not line.strip():
            continue
        mode, _obj, stage_path = line.split(maxsplit=2)
        _stage, rel = stage_path.split("\t", 1)
        if mode != "100755":
            continue
        if rel.startswith("packages/") or "/tests/" in rel:
            continue
        if _find_python_migration_exception(repo_root, "executable_python", rel, rel) is None:
            errors.append(f"executable python outside packages/: {rel}")

    for rel in _git_ls_files(repo_root, ["*.sh"]):
        if rel.startswith("docker/") or rel.startswith("packages/"):
            continue
        if _find_python_migration_exception(repo_root, "shell_location", rel, "") is None:
            errors.append(f"shell script outside docker/ or packages/: {rel}")
    return (0 if not errors else 1), errors


def check_atlas_scripts_cli_contract(repo_root: Path) -> tuple[int, list[str]]:
    cli = repo_root / "bin/atlasctl"
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    expected_version = ""
    for ln in pyproject.read_text(encoding="utf-8").splitlines():
        stripped = ln.strip()
        if stripped.startswith("version = "):
            expected_version = stripped.split("=", 1)[1].strip().strip('"').strip("'")
            break
    errs: list[str] = []
    h1 = subprocess.run([str(cli), "--help"], cwd=repo_root, text=True, capture_output=True, check=False)
    h2 = subprocess.run([str(cli), "--help"], cwd=repo_root, text=True, capture_output=True, check=False)
    if h1.returncode != 0 or h2.returncode != 0:
        errs.append("atlasctl --help must exit 0")
    if h1.stdout != h2.stdout:
        errs.append("atlasctl --help output is not deterministic")
    v = subprocess.run([str(cli), "--version"], cwd=repo_root, text=True, capture_output=True, check=False)
    if v.returncode != 0:
        errs.append("atlasctl --version must exit 0")
    else:
        out = (v.stdout or v.stderr).strip()
        if expected_version and expected_version not in out:
            errs.append(f"atlasctl version mismatch: expected {expected_version}, got `{out}`")
    return (0 if not errs else 1), errs


def check_atlasctl_boundaries(repo_root: Path) -> tuple[int, list[str]]:
    from ..layout.boundary_check import check_boundaries

    violations = check_boundaries(repo_root)
    errors = [f"{v.file}:{v.line} disallowed import {v.source} -> {v.target}" for v in violations]
    return (0 if not errors else 1), errors


def generate_scripts_sbom(repo_root: Path, lock_rel: str, out_rel: str) -> tuple[int, list[str]]:
    lock = repo_root / lock_rel
    lines = [ln.strip() for ln in lock.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.startswith("#")]
    packages = []
    for item in lines:
        name, version = item.split("==", 1)
        packages.append({"name": name, "version": version, "purl": f"pkg:pypi/{name}@{version}"})
    payload = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source_lock": lock.as_posix(),
        "package_count": len(packages),
        "packages": packages,
    }
    out = repo_root / out_rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0, [out_rel]


def check_root_bin_shims(repo_root: Path) -> tuple[int, list[str]]:
    bin_dir = repo_root / "bin"
    if not bin_dir.exists():
        return 0, []
    max_lines = 30
    allowed = re.compile(
        r"^(#!|# DEPRECATED:|# Migration:|echo \"DEPRECATED: .*\" >&2|set -euo pipefail|set -eu|ROOT=|PYTHONPATH=|\s*exec python3 -m atlasctl\.cli \"\$@\"|exec \".*/bijux-atlas\" make (explain|graph|help) \"\$@\"|\s*$)"
    )
    errors: list[str] = []
    for path in sorted(bin_dir.iterdir()):
        if path.name == "README.md" or not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        rel = path.relative_to(repo_root).as_posix()
        if len(lines) > max_lines:
            errors.append(f"{rel} exceeds {max_lines} lines")
        for idx, line in enumerate(lines, 1):
            if not allowed.match(line):
                errors.append(f"{rel}:{idx}: non-shim logic is forbidden")
                break
    return (0 if not errors else 1), errors


def check_effects_lint(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    planner_files = [
        repo_root / "crates/bijux-atlas-query/src/planner/mod.rs",
        repo_root / "crates/bijux-atlas-query/src/filters.rs",
        repo_root / "crates/bijux-atlas-query/src/cost.rs",
        repo_root / "crates/bijux-atlas-query/src/limits.rs",
    ]
    forbidden = ("rusqlite", "reqwest", "std::fs", "tokio::net", "std::process")
    for path in planner_files:
        if not path.exists():
            errors.append(f"missing planner file: {path.relative_to(repo_root)}")
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for pat in forbidden:
            if re.search(rf"\b{re.escape(pat)}\b", text):
                errors.append(f"forbidden `{pat}` in {path.relative_to(repo_root)}")
    http_root = repo_root / "crates/bijux-atlas-server/src/http"
    if http_root.exists():
        for path in sorted(http_root.rglob("*.rs")):
            if path.name == "effects_adapters.rs":
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            if re.search(r"std::fs::|use std::fs::|File::open\(", text):
                errors.append(f"raw fs IO forbidden in {path.relative_to(repo_root)}")
            if re.search(
                r"runtime::dataset_cache_manager_(maintenance|storage)|crate::runtime::dataset_cache_manager_(maintenance|storage)",
                text,
            ):
                errors.append(f"http mapping must not import runtime effect internals in {path.relative_to(repo_root)}")
    return (0 if not errors else 1), errors


def check_naming_intent_lint(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in sorted((repo_root / "crates").rglob("*")):
        if not path.is_file():
            continue
        name = path.name
        if name == "helpers.rs" or name.endswith("_helpers.rs"):
            errors.append(path.relative_to(repo_root).as_posix())
    return (0 if not errors else 1), errors
