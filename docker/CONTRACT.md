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
