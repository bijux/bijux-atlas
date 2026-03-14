# Trust Boundaries

The trust boundaries are text-first so the model remains reviewable in git.

## Boundary map

1. External client to atlas HTTP surface:
   Requests cross from untrusted callers into the runtime API and observability endpoints.
2. Operator control-plane to local and cluster subprocesses:
   `bijux-dev-atlas` crosses into `kubectl`, `helm`, `kind`, and filesystem mutation.
3. Runtime to dependency services:
   Atlas crosses from its pod boundary to Redis, object storage, catalog, and telemetry collectors.
4. Release production to evidence consumers:
   Generated artifacts cross from build-time control-plane execution to downstream operators and reviewers.
