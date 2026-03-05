-- named_query: record_count
SELECT COUNT(*) AS record_count FROM combined_release;

-- named_query: sample_rows
SELECT * FROM combined_release LIMIT 10;
