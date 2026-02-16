# User Stories

Top real-world questions Atlas must answer:

1. For release 110, what is the gene count for homo_sapiens GRCh38?
2. What are the genes in chr1:100000-200000 for release 110?
3. What is the exact gene record for `ENSG00000141510` in release 110?
4. Which genes start with `BRCA` in homo_sapiens GRCh38?
5. Which genes have biotype `protein_coding` in a dataset?
6. Which dataset versions are currently published for mus_musculus GRCm39?
7. Is dataset `110/homo_sapiens/GRCh38` currently cached on this server?
8. Is the cached dataset checksum-verified and safe to serve?
9. What schema version is used by this dataset SQLite artifact?
10. What QC summary was produced at ingest time (genes/transcripts/contigs)?
11. Why did a query get rejected (limit/span/cost guard)?
12. Which query class was applied (cheap/medium/heavy)?
13. Which policy keys were used to extract gene name/biotype in this build?
14. Can I fetch dataset metadata directly from dimensions without listing all datasets?
15. Does this release support stable pagination across repeated reads?
16. What happens when catalog/store are down but dataset is cached?
17. Can I detect API and artifact compatibility before upgrade?
18. How do I validate a dataset before publish?
19. Which datasets are pinned and protected from eviction?
20. Where do I find SLO targets and production acceptance criteria?
