# Docker Build Surface

This repository currently ships one runtime image built from root `Dockerfile`.

Rules:
- Keep image build/release commands behind make targets.
- If additional images are introduced, place Dockerfiles under `docker/` and document each image contract here.

Canonical commands:

```bash
make docker-build
make docker-smoke
```
