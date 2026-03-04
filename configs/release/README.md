# Release Configs

This directory contains release policy and release process config inputs consumed by `bijux-dev-atlas` release commands.

- `version-policy.json`: semantic versioning, prerelease tag allowlist, and changelog policy used by release validation commands.
- `reproducibility-policy.json`: canonical build environment, hash-match, and reproducibility evidence requirements.
- `configs/schema/release-version-policy.schema.json`: schema for validating `version-policy.json`.
- `configs/schema/release-reproducibility-policy.schema.json`: schema for validating `reproducibility-policy.json`.
