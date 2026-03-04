# Observability FAQ

## Which endpoint should an ingress probe use?

Use readiness (`/ready` or `/readyz`) for traffic gating and liveliness (`/live` or `/healthz`) for restart policy.

## What is the fastest way to collect incident evidence?

Run `bijux-dev-atlas system debug diagnostics --format json` and archive the artifact output.

## Why keep both `atlas_*` and `bijux_*` metrics?

`atlas_*` is canonical; `bijux_*` aliases exist only for compatibility windows.
