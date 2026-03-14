# Effects Policy

Allowed effects in `bijux-atlas-core`:
- Pure compute and memory allocation.
- Environment-variable reads only for config-path helpers.

Disallowed effects:
- Filesystem read/write.
- Network I/O.
- Process spawn.
- Time-based nondeterministic behavior in canonicalization/error paths.

No allocation surprises:
- APIs that allocate return owned values explicitly (`String`, `Vec<u8>`).
- Canonicalization functions avoid hidden global caches.
- Any potentially large allocation must be input-size proportional and documented.
