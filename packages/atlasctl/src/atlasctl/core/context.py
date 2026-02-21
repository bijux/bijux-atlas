from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal

from .adapters.git import read_git_context
from .errors import ScriptError
from .env import getenv
from .isolation import require_isolate_env
from .paths import evidence_root_path, find_repo_root, run_dir_root_path, scripts_artifact_root_path

OutputFormat = Literal["text", "json"]
NetworkMode = Literal["allow", "forbid"]


@dataclass(frozen=True)
class RunContext:
    run_id: str
    profile: str
    repo_root: Path
    evidence_root: Path
    scripts_artifact_root: Path
    run_dir: Path
    output_format: OutputFormat
    network_mode: NetworkMode
    verbose: bool
    quiet: bool
    require_clean_git: bool
    no_network: bool
    log_json: bool
    git_sha: str
    git_dirty: bool

    @property
    def scripts_root(self) -> Path:
        return (self.repo_root / "artifacts/atlasctl").resolve()

    @property
    def fs(self):  # noqa: ANN201
        from . import fs as fs_module

        return fs_module

    @property
    def exec(self):  # noqa: ANN201
        from . import exec as exec_module

        return exec_module

    @property
    def env(self):  # noqa: ANN201
        from . import env as env_module

        return env_module

    @property
    def process(self):  # noqa: ANN201
        from . import process as process_module

        return process_module

    @property
    def network(self):  # noqa: ANN201
        from . import network as network_module

        return network_module

    def require_isolate(self, env: dict[str, str] | None = None) -> None:
        ok, message = require_isolate_env(env)
        if not ok:
            raise ScriptError(f"isolate-required: {message}", 1)

    @classmethod
    def from_args(
        cls,
        run_id: str | None,
        evidence_root: str | None,
        profile: str | None,
        no_network: bool,
        output_format: OutputFormat = "text",
        network_mode: NetworkMode = "allow",
        run_dir: str | None = None,
        verbose: bool = False,
        quiet: bool = False,
        require_clean_git: bool = False,
        log_json: bool = False,
    ) -> "RunContext":
        repo_root = find_repo_root()
        git_ctx = read_git_context(repo_root)
        default_run = f"atlas-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}-{git_ctx.sha}"
        resolved_run_id = run_id or getenv("RUN_ID", default_run)
        resolved_profile = profile or getenv("PROFILE", "local")
        resolved_evidence_root = evidence_root_path(repo_root, evidence_root or getenv("EVIDENCE_ROOT"))
        scripts_artifact_root = scripts_artifact_root_path(
            repo_root,
            getenv("BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT", f"artifacts/atlasctl/run/{resolved_run_id}"),
        )
        run_dir_root = run_dir_root_path(repo_root, resolved_evidence_root, run_dir)
        resolved_network_mode: NetworkMode = "forbid" if no_network else network_mode
        resolved_no_network = resolved_network_mode == "forbid"
        return cls(
            run_id=resolved_run_id,
            profile=resolved_profile,
            repo_root=repo_root,
            evidence_root=resolved_evidence_root,
            scripts_artifact_root=scripts_artifact_root,
            run_dir=run_dir_root / resolved_run_id,
            output_format=output_format,
            network_mode=resolved_network_mode,
            verbose=verbose,
            quiet=quiet,
            require_clean_git=require_clean_git,
            no_network=resolved_no_network,
            log_json=log_json,
            git_sha=git_ctx.sha,
            git_dirty=git_ctx.is_dirty,
        )
