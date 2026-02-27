# Docker Contract

- Owner: `bijux-atlas-platform`
- Enforced by: `bijux dev atlas contracts docker`

## Contract Registry

### DOCKER-000 docker directory contract

Tests:
- `docker.dir.allowed_markdown` (static, Pure): only README.md and CONTRACT.md are allowed markdown files
- `docker.dir.no_contracts_subdir` (static, Pure): docker/contracts subdirectory is forbidden
- `docker.dir.dockerfiles_location` (static, Pure): Dockerfiles must be under docker/images/**

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

### DOCKER-034 image manifest schema

Tests:
- `docker.images.manifest_schema_valid` (static, Pure): docker/images.manifest.json is schema-valid and non-empty

### DOCKER-035 image manifest completeness

Tests:
- `docker.images.manifest_matches_dockerfiles` (static, Pure): image manifest matches the actual Dockerfile set

### DOCKER-036 image build matrix

Tests:
- `docker.build_matrix.defined` (static, Pure): docker/build-matrix.json covers every manifest image

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

### DOCKER-037 manifest image builds

Tests:
- `docker.effect.build_each_manifest_image` (effect, Subprocess): each image declared in the manifest builds successfully

### DOCKER-038 manifest image smoke

Tests:
- `docker.effect.smoke_each_manifest_image` (effect, Subprocess): each image declared in the manifest passes its smoke command

### DOCKER-039 ci build pull policy

Tests:
- `docker.effect.ci_build_uses_pull_false` (effect, Subprocess): CI image builds use --pull=false for deterministic base resolution

### DOCKER-040 build metadata artifact

Tests:
- `docker.effect.build_metadata_written` (effect, Subprocess): image build metadata is recorded as JSON in artifacts

### DOCKER-041 manifest sbom coverage

Tests:
- `docker.effect.sbom_for_each_manifest_image` (effect, Subprocess): each image declared in the manifest produces an SBOM

### DOCKER-042 scan artifact threshold

Tests:
- `docker.effect.scan_output_and_threshold` (effect, Network): scanner output is stored and respects the configured severity threshold

### DOCKER-043 vulnerability allowlist discipline

Tests:
- `docker.effect.no_high_critical_without_allowlist` (effect, Network): HIGH and CRITICAL vulnerabilities require an explicit allowlist with justification

### DOCKER-044 pip install hash pinning

Tests:
- `docker.run.no_pip_install_without_hashes` (static, Pure): pip install uses --require-hashes or a lock strategy

### DOCKER-045 cargo install version pinning

Tests:
- `docker.run.no_cargo_install_without_version` (static, Pure): cargo install pins an explicit version

### DOCKER-046 go install latest forbidden

Tests:
- `docker.run.no_go_install_latest` (static, Pure): go install does not use @latest

### DOCKER-047 docker markdown boundary

Tests:
- `docker.docs.markdown_surface_only_root_docs` (static, Pure): docker contains only README.md and CONTRACT.md as markdown

### DOCKER-048 contract document generation

Tests:
- `docker.contract_doc.generated_match` (static, Pure): docker CONTRACT document matches generated registry and mapping

### DOCKER-049 contract registry export

Tests:
- `docker.registry.export_matches_generated` (static, Pure): docker/docker.contracts.json matches generated registry output

### DOCKER-050 contract gate map export

Tests:
- `docker.gate_map.matches_generated` (static, Pure): docker contract gate map matches generated output

### DOCKER-051 exceptions registry schema

Tests:
- `docker.exceptions.schema_valid` (static, Pure): docker/exceptions.json uses the expected strict schema

### DOCKER-052 exceptions minimal entries

Tests:
- `docker.exceptions.minimal_entries` (static, Pure): each docker exception cites a contract id, expiry date, and justification

### DOCKER-053 scan profile policy

Tests:
- `docker.scan.profile_policy` (static, Pure): docker policy defines local scan skip and ci scan enforcement

### DOCKER-054 runtime engine policy

Tests:
- `docker.runtime.engine_policy` (static, Pure): docker policy explicitly declares docker-only runtime engine

### DOCKER-055 airgap build policy

Tests:
- `docker.build.airgap_policy_stub` (static, Pure): docker policy carries an explicit airgap build stub

### DOCKER-056 multi-registry push policy

Tests:
- `docker.push.multi_registry_policy_stub` (static, Pure): docker policy carries an explicit multi-registry push stub

### DOCKER-057 downloaded asset digest pinning

Tests:
- `docker.run.downloaded_assets_are_verified` (static, Pure): downloaded assets are paired with an in-instruction checksum verification

### DOCKER-058 vendored binary declaration

Tests:
- `docker.vendored_binaries.allowlisted` (static, Pure): vendored binary artifacts are explicitly allowlisted

### DOCKER-059 curl pipe shell forbidden

Tests:
- `docker.run.no_curl_pipe_shell` (static, Pure): curl or wget pipelines must not feed shell interpreters

### DOCKER-060 dockerfile formatting

Tests:
- `docker.dockerfiles.canonical_whitespace` (static, Pure): dockerfiles avoid tabs and trailing whitespace

## Mapping

| Contract | Gate | Command |
| --- | --- | --- |
| `DOCKER-000` | `docker.contract.docker_000` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-003` | `docker.contract.docker_003` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-004` | `docker.contract.docker_004` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-006` | `docker.contract.docker_006` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-007` | `docker.contract.docker_007` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-008` | `docker.contract.docker_008` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-009` | `docker.contract.docker_009` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-010` | `docker.contract.docker_010` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-011` | `docker.contract.docker_011` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-012` | `docker.contract.docker_012` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-013` | `docker.contract.docker_013` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-014` | `docker.contract.docker_014` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-015` | `docker.contract.docker_015` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-016` | `docker.contract.docker_016` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-017` | `docker.contract.docker_017` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-018` | `docker.contract.docker_018` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-019` | `docker.contract.docker_019` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-020` | `docker.contract.docker_020` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-021` | `docker.contract.docker_021` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-022` | `docker.contract.docker_022` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-023` | `docker.contract.docker_023` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-024` | `docker.contract.docker_024` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-025` | `docker.contract.docker_025` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-026` | `docker.contract.docker_026` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-027` | `docker.contract.docker_027` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-028` | `docker.contract.docker_028` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-029` | `docker.contract.docker_029` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-030` | `docker.contract.docker_030` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-031` | `docker.contract.docker_031` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-032` | `docker.contract.docker_032` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-033` | `docker.contract.docker_033` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-034` | `docker.contract.docker_034` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-035` | `docker.contract.docker_035` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-036` | `docker.contract.docker_036` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-100` | `docker.contract.docker_100` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-101` | `docker.contract.docker_101` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-102` | `docker.contract.docker_102` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-103` | `docker.contract.docker_103` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-network --allow-docker-daemon` |
| `DOCKER-037` | `docker.contract.docker_037` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-038` | `docker.contract.docker_038` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-039` | `docker.contract.docker_039` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-040` | `docker.contract.docker_040` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-041` | `docker.contract.docker_041` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-docker-daemon` |
| `DOCKER-042` | `docker.contract.docker_042` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-network --allow-docker-daemon` |
| `DOCKER-043` | `docker.contract.docker_043` | `bijux dev atlas contracts docker --mode effect --allow-subprocess --allow-network --allow-docker-daemon` |
| `DOCKER-044` | `docker.contract.docker_044` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-045` | `docker.contract.docker_045` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-046` | `docker.contract.docker_046` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-047` | `docker.contract.docker_047` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-048` | `docker.contract.docker_048` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-049` | `docker.contract.docker_049` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-050` | `docker.contract.docker_050` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-051` | `docker.contract.docker_051` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-052` | `docker.contract.docker_052` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-053` | `docker.contract.docker_053` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-054` | `docker.contract.docker_054` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-055` | `docker.contract.docker_055` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-056` | `docker.contract.docker_056` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-057` | `docker.contract.docker_057` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-058` | `docker.contract.docker_058` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-059` | `docker.contract.docker_059` | `bijux dev atlas contracts docker --mode static` |
| `DOCKER-060` | `docker.contract.docker_060` | `bijux dev atlas contracts docker --mode static` |

## Rule

- Contract ID or test ID missing from this document means it does not exist.
