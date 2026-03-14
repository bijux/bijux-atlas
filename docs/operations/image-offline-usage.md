# Offline Image Usage

1. Pre-pull required image digests in connected environment.
2. Export and transfer image archives with checksums.
3. Import into offline registry or local daemon.
4. Verify digests against `ops/release/images/image-artifact-manifest.v0.1.json`.
5. Run `bijux-dev-atlas runtime self-check --format json` before serving traffic.
