# Configs Contract

- `CONFIGS-001`: Only `configs/README.md` and `configs/CONTRACT.md` may exist as markdown at the `configs/` root. Enforced by: `configs.root.only_root_docs`.
- `CONFIGS-002`: Every governed config file must be covered by the configs registry or an explicit registry exclusion. Enforced by: `configs.registry.no_undocumented_files`.
- `CONFIGS-003`: Governed config paths must stay within the configured depth budget. Enforced by: `configs.layout.depth_budget`.
- `CONFIGS-004`: Internal-only config surfaces must use explicit internal classification and allowed internal naming homes. Enforced by: `configs.naming.internal_surface`.
- `CONFIGS-005`: Every configs group must declare a non-empty owner in the configs registry. Enforced by: `configs.registry.owner_complete`.
- `CONFIGS-006`: JSON config files must either be schema files, map to schema coverage, or be explicitly excluded. Enforced by: `configs.schema.coverage`.
- `CONFIGS-007`: Tool dependency configs must carry lock companions. Enforced by: `configs.lockfiles.required_pairs`.
- `CONFIGS-008`: Registry ownership must not overlap; one governed file maps to one authoritative registry home. Enforced by: `configs.registry.no_overlap`.
- `CONFIGS-009`: Generated config paths are forbidden unless explicitly modeled in the registry. Enforced by: `configs.generated.authored_boundary`.
- `CONFIGS-010`: Documented contracts must match the enforced configs contract set. Enforced by: `configs.contracts.no_policy_theater`.
- `CONFIGS-011`: The configs registry must cover the full `configs/` file surface. Enforced by: `configs.registry.complete_surface`.
- `CONFIGS-012`: No non-excluded config file may be orphaned outside the registry. Enforced by: `configs.registry.no_orphans`.
- `CONFIGS-013`: Registry entries must resolve to real files. Enforced by: `configs.registry.no_dead_entries`.
- `CONFIGS-014`: The configs registry group count must stay inside the declared budget. Enforced by: `configs.registry.group_budget`.
- `CONFIGS-015`: Included group file paths must stay inside the declared per-group depth budget. Enforced by: `configs.registry.group_depth_budget`.
- `CONFIGS-016`: Every group must classify covered files as public, internal, generated, or schema-backed. Enforced by: `configs.registry.visibility_classification`.
