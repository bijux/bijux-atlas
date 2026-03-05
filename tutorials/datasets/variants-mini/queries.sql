-- named_query: record_count
SELECT COUNT(*) AS record_count FROM variants_mini;

-- named_query: sample_rows
SELECT * FROM variants_mini LIMIT 10;
