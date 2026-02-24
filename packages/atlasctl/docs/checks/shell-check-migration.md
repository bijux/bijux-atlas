# Shell Check Migration Inventory

This table tracks legacy shell check intents and their atlasctl-native Python implementations.

| Legacy shell check intent | Python-native check implementation | Status |
|---|---|---|
| `no-empty-dirs.sh` | `packages/atlasctl/src/atlasctl/checks/tools/repo_domain/enforcement/package_hygiene.py` (`check_no_empty_dirs_or_pointless_nests`) | migrated |
| `naming.sh` | `packages/atlasctl/src/atlasctl/checks/tools/repo_domain/domains/policies.py` (`check_naming_intent_lint`) and `packages/atlasctl/src/atlasctl/checks/layout/scripts/check_script_naming_convention.py` | migrated |
| `no-bin-symlinks.sh` | `packages/atlasctl/src/atlasctl/checks/layout/domains/root/check_symlink_policy.py` and `packages/atlasctl/src/atlasctl/checks/tools/repo_domain/contracts/required_proof.py` | migrated |
| `no-scripts-bin-dir.sh` | `packages/atlasctl/src/atlasctl/checks/tools/repo_domain/domains/scripts_dir.py` and `packages/atlasctl/src/atlasctl/checks/tools/repo_domain/root_forbidden_paths.py` | migrated |
| `lint/**/*.sh` checks | No `.sh` files remain under `packages/atlasctl/src/atlasctl/checks` | migrated |

## Migration Completeness Gate

The `checks` structure contract now fails if any `.sh` file exists under `packages/atlasctl/src/atlasctl/checks`.
It also fails on duplicate check IDs in generated registry artifacts.
