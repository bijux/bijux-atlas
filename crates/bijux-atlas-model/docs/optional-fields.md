# Optional Fields Rules

API representation policy:
- For v1 machine contracts, optional fields should be represented consistently per endpoint.
- Use `null` when field presence is structurally expected.
- Use omission only when endpoint contract explicitly allows absent field.

Model type support:
- `OptionalFieldPolicy` documents explicit encoding choice.
- `OptionalFieldPolicy::apply_to_json_map` enforces deterministic behavior for serializers.
