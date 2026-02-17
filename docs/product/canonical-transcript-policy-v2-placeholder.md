# Canonical Transcript Policy (v2 Placeholder)

Status: placeholder only.

Atlas v1 intentionally does not select or expose a canonical transcript per gene.
A future v2 policy may define this using explicit ranking rules and provenance.

Until then:

- No `canonical_transcript_id` field is emitted.
- Clients must not assume first transcript in a page is canonical.
