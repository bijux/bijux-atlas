# Docker Build Network Policy

## Scope

This policy applies to Dockerfiles under `docker/images/**`.

## Rule

Docker builds must avoid outbound network access except for explicit, audited steps that are required to assemble the image.

## Approved exceptions for `docker/images/runtime/Dockerfile`

1. `apt-get update` and `apt-get install` in the builder stage.
2. `cargo build --locked` dependency resolution in the builder stage.

These are temporary bootstrap exceptions for reproducible build inputs. Any additional networked build step requires a policy update in this file and review.

## Enforcement

1. `crates/bijux-dev-atlas/tests/dockerfile_contracts.rs` validates Dockerfile source path and base-image pinning policy contracts.
2. CI must run Docker image build and smoke checks in `.github/workflows/ci-pr.yml`.
