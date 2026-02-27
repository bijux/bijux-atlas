# Docker Contract

- Owner: `bijux-atlas-platform`
- Enforced by: `bijux dev atlas contracts docker`

## Contract Registry

### DOCKER-000 docker directory contract

Tests:
- `docker.dir.allowed_markdown`: only README.md and CONTRACT.md are allowed markdown files
- `docker.dir.no_contracts_subdir`: docker/contracts subdirectory is forbidden
- `docker.dir.dockerfiles_location`: Dockerfiles must be under docker/images/**
- `docker.contract_doc.generated_match`: docker CONTRACT document matches generated registry

### DOCKER-003 root Dockerfile policy

Tests:
- `docker.root_dockerfile.symlink_or_absent`: root Dockerfile is symlink or absent
- `docker.root_dockerfile.target_runtime`: root Dockerfile symlink target is runtime Dockerfile

### DOCKER-004 dockerfile location policy

Tests:
- `docker.dockerfiles.under_images_only`: Dockerfiles are only under docker/images/**
- `docker.dockerfiles.filename_convention`: Dockerfiles follow filename convention

### DOCKER-006 forbidden tags policy

Tests:
- `docker.from.no_latest`: FROM does not use latest
- `docker.from.no_floating_tags`: FROM does not use floating tags unless allowlisted

### DOCKER-007 digest pinning policy

Tests:
- `docker.from.digest_required`: FROM images require digest pin unless allowlisted
- `docker.from.repo_digest_format`: FROM digest format is valid

### DOCKER-008 required labels policy

Tests:
- `docker.labels.required_present`: required OCI labels are present
- `docker.labels.required_nonempty`: required OCI labels are non-empty

### DOCKER-009 build args defaults policy

Tests:
- `docker.args.defaults_present`: required ARG directives include defaults
- `docker.args.required_declared`: required ARG directives are declared

### DOCKER-010 forbidden pattern policy

Tests:
- `docker.pattern.no_curl_pipe_sh`: RUN curl|sh is forbidden
- `docker.pattern.no_add_remote_url`: ADD remote URL is forbidden

### DOCKER-011 copy source policy

Tests:
- `docker.copy.sources_exist`: COPY sources must exist
- `docker.copy.no_absolute_sources`: COPY absolute sources are forbidden
- `docker.copy.no_parent_traversal`: COPY sources must not use parent traversal

## Rule

- Contract ID or test ID missing from this document means it does not exist.
