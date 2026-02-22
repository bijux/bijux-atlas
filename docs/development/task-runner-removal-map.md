# Xtask Removal Map

`xtask` was removed. Use `atlasctl` commands:

- `xtask format-contracts` -> `./bin/atlasctl gen contracts`
- `xtask generate-contracts` -> `./bin/atlasctl gen contracts`
- `xtask check-contracts` -> `./bin/atlasctl check configs`
- `xtask scan-relaxations` -> `./bin/atlasctl policies scan-rust-relaxations`

