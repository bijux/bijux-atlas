# bijux-versioning

`bijux-versioning` is the shared Rust helper crate that derives runtime version
identity from git tags, package metadata, and explicit overrides.

It exists so Bijux binaries can keep one versioning contract instead of
re-implementing tag parsing and display formatting in each executable crate.

## What It Provides

- tag-aware runtime version resolution
- display-version formatting for end-user command surfaces
- semver-compatible derived versions for untagged worktree builds
- release-line helpers for control-plane and release automation code

## Intended Use

This crate is an internal building block for Bijux Rust products. It is
published so first-party binaries can depend on it through normal Cargo release
flows, but it is not positioned as a standalone end-user product.
