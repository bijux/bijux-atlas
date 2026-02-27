# Docker Contract

- Owner: `bijux-atlas-platform`
- Enforced by: `bijux dev atlas contracts docker`

## Contract Registry

### DOCKER-000 docker directory contract

Tests:
- `docker.dir.allowed_markdown` (static, Pure): only README.md and CONTRACT.md are allowed markdown files
- `docker.dir.no_contracts_subdir` (static, Pure): docker/contracts subdirectory is forbidden
- `docker.dir.dockerfiles_location` (static, Pure): Dockerfiles must be under docker/images/**
- `docker.contract_doc.generated_match` (static, Pure): docker CONTRACT document matches generated registry

### DOCKER-003 root Dockerfile policy

Tests:
- `docker.root_dockerfile.symlink_or_absent` (static, Pure): root Dockerfile is symlink or absent
- `docker.root_dockerfile.target_runtime` (static, Pure): root Dockerfile symlink target is runtime Dockerfile

### DOCKER-004 dockerfile location policy

Tests:
- `docker.dockerfiles.under_images_only` (static, Pure): Dockerfiles are only under docker/images/**
- `docker.dockerfiles.filename_convention` (static, Pure): Dockerfiles follow filename convention

### DOCKER-006 forbidden tags policy

Tests:
- `docker.from.no_latest` (static, Pure): FROM does not use latest
- `docker.from.no_floating_tags` (static, Pure): FROM does not use floating tags unless allowlisted

### DOCKER-007 digest pinning policy

Tests:
- `docker.from.digest_required` (static, Pure): FROM images require digest pin unless allowlisted
- `docker.from.repo_digest_format` (static, Pure): FROM digest format is valid

### DOCKER-008 required labels policy

Tests:
- `docker.labels.required_present` (static, Pure): required OCI labels are present
- `docker.labels.required_nonempty` (static, Pure): required OCI labels are non-empty

### DOCKER-009 build args defaults policy

Tests:
- `docker.args.defaults_present` (static, Pure): required ARG directives include defaults
- `docker.args.required_declared` (static, Pure): required ARG directives are declared

### DOCKER-010 forbidden pattern policy

Tests:
- `docker.pattern.no_curl_pipe_sh` (static, Pure): RUN curl|sh is forbidden
- `docker.pattern.no_add_remote_url` (static, Pure): ADD remote URL is forbidden

### DOCKER-011 copy source policy

Tests:
- `docker.copy.sources_exist` (static, Pure): COPY sources must exist
- `docker.copy.no_absolute_sources` (static, Pure): COPY absolute sources are forbidden
- `docker.copy.no_parent_traversal` (static, Pure): COPY sources must not use parent traversal

### DOCKER-012 required images exist

Tests:
- `docker.images.required_exist` (static, Pure): required image directories include Dockerfile

### DOCKER-013 forbidden extra images

Tests:
- `docker.images.forbidden_extra` (static, Pure): docker image directories are allowlisted

### DOCKER-014 branch-like tags forbidden

Tests:
- `docker.from.no_branch_like_tags` (static, Pure): FROM does not use main, master, edge, or nightly tags

### DOCKER-015 base image allowlist

Tests:
- `docker.from.allowlisted_base_images` (static, Pure): FROM images are declared in docker/bases.lock

### DOCKER-016 base image lock digest

Tests:
- `docker.from.digest_matches_lock` (static, Pure): FROM image digests match docker/bases.lock

### DOCKER-017 from arg defaults

Tests:
- `docker.from.args_have_defaults` (static, Pure): ARG values referenced by FROM have defaults

### DOCKER-018 from platform override

Tests:
- `docker.from.no_platform_override` (static, Pure): FROM does not use --platform unless policy allows it

### DOCKER-019 shell instruction policy

Tests:
- `docker.shell.explicit_policy` (static, Pure): Dockerfile SHELL usage follows docker policy

### DOCKER-020 package manager cleanup

Tests:
- `docker.run.package_manager_cleanup` (static, Pure): package manager installs include deterministic cleanup

### DOCKER-021 runtime non-root user

Tests:
- `docker.runtime.non_root` (static, Pure): final runtime stage uses a non-root user

### DOCKER-022 final stage user declaration

Tests:
- `docker.final_stage.user_required` (static, Pure): final stage declares USER explicitly

### DOCKER-023 final stage workdir

Tests:
- `docker.final_stage.workdir_required` (static, Pure): final stage declares WORKDIR explicitly

### DOCKER-024 final stage process entry

Tests:
- `docker.final_stage.entrypoint_or_cmd_required` (static, Pure): final stage declares ENTRYPOINT or CMD

### DOCKER-025 release labels contract

Tests:
- `docker.labels.contract_fields` (static, Pure): release labels include provenance, timestamp, and license fields

### DOCKER-026 secret copy guard

Tests:
- `docker.copy.no_secrets` (static, Pure): COPY does not include secret-like files

### DOCKER-027 add instruction forbidden

Tests:
- `docker.add.forbidden` (static, Pure): Dockerfiles use COPY instead of ADD unless explicitly allowlisted

### DOCKER-028 multistage build required

Tests:
- `docker.build.multistage_required` (static, Pure): builds that compile artifacts use a builder stage

### DOCKER-029 dockerignore required entries

Tests:
- `docker.ignore.required_entries` (static, Pure): .dockerignore includes deterministic exclusions

### DOCKER-030 reproducible build args

Tests:
- `docker.args.repro_build_args` (static, Pure): reproducible build args are declared

### DOCKER-031 final stage network isolation

Tests:
- `docker.final_stage.no_network` (static, Pure): final stage does not fetch over the network

### DOCKER-032 final stage package manager isolation

Tests:
- `docker.final_stage.no_package_manager` (static, Pure): final stage does not run package managers

### DOCKER-033 image smoke manifest

Tests:
- `docker.images.smoke_manifest` (static, Pure): each Docker image is listed with a smoke command in docker/images.manifest.json

### DOCKER-100 build succeeds

Tests:
- `docker.build.runtime_image` (effect, Subprocess): runtime image build succeeds

### DOCKER-101 runtime smoke checks

Tests:
- `docker.smoke.version` (effect, Subprocess): runtime image prints version
- `docker.smoke.help` (effect, Subprocess): runtime image prints help

### DOCKER-102 sbom generated

Tests:
- `docker.sbom.generated` (effect, Subprocess): syft generates a JSON SBOM

### DOCKER-103 scan passes policy

Tests:
- `docker.scan.severity_threshold` (effect, Network): trivy scan passes configured severity threshold

## Rule

- Contract ID or test ID missing from this document means it does not exist.
