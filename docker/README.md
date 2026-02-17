# Docker Build Surface

This repository currently ships one runtime image built from `docker/Dockerfile`.

Rules:
- Keep image build/release commands behind make targets.
- Root `Dockerfile` is a compatibility symlink only.
- If additional images are introduced, place Dockerfiles under `docker/` and document each image contract here.

Canonical commands:

```bash
make docker-build
make docker-smoke
```
