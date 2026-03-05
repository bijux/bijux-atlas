-- named_query: record_count
SELECT COUNT(*) AS record_count FROM transcripts_baseline;

-- named_query: sample_rows
SELECT * FROM transcripts_baseline LIMIT 10;
