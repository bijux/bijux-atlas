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


from .repo.native_runtime import (
    check_python_migration_exceptions_expiry,
    check_bin_entrypoints,
    check_python_lock,
    check_scripts_lock_sync,
    check_no_adhoc_python,
    check_no_direct_python_invocations,
    check_no_direct_bash_invocations,
    check_docs_no_ops_generated_run_paths,
    check_no_ops_generated_placeholder,
    check_ops_examples_immutable,
    check_invocation_parity,
    check_scripts_surface_docs_drift,
    check_script_errors,
    check_script_write_roots,
    check_script_tool_guards,
    check_script_shim_expiry,
    check_script_shims_minimal,
    check_venv_location_policy,
    check_python_runtime_artifacts,
    check_repo_script_boundaries,
    check_atlas_scripts_cli_contract,
    check_atlasctl_boundaries,
    generate_scripts_sbom,
    check_root_bin_shims,
    check_effects_lint,
    check_naming_intent_lint,
)
