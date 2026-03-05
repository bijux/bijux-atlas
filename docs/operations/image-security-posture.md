# Image Security Posture

- Base images are digest pinned and checked against `docker/bases.lock`.
- Runtime image executes as `nonroot`.
- Release requires SBOM and provenance artifacts.
- Vulnerability reports are produced as informational evidence first, then can be made blocking.
- Runtime hardening contract blocks broad write-permission patterns.
