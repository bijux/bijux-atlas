# Docker

Container build, validation, and release behavior is defined by executable release and policy surfaces.

## Intent

- Keep docker policy minimal, machine-readable, and enforceable.
- Keep docker docs limited to this file plus the machine-readable policy and manifest artifacts.
- Route operational behavior through `bijux dev atlas release images ...` and the runtime Dockerfile itself.

## Canonical Files

- `ops/docker/README.md`
- `ops/docker/policy.json`
- `ops/docker/images.manifest.json`
- `ops/docker/images/runtime/Dockerfile`

Root `Dockerfile` is a shim symlink to the canonical runtime Dockerfile.

## Canonical Commands

```bash
bijux dev atlas release images manifest-verify
bijux dev atlas release images validate-base-digests
bijux dev atlas release images smoke-verify
```

## Artifacts

Docker gate artifacts are written under:

`artifacts/<run_id>/...`

## Enforcement

Policy meaning lives in executable release surfaces, not prose.
