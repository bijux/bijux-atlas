# Transcript Ordering Rules

Transcript query ordering in v1 is stable and deterministic:

1. `seqid` ascending
2. `start` ascending
3. `transcript_id` ascending

Pagination cursor for transcript list uses last `transcript_id` from the page.
