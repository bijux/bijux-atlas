// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn large_manifest() -> ArtifactManifest {
    let mut manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64),
        ),
        ManifestStats::new(100_000, 200_000, 24),
    );

    manifest.input_hashes.gff3_sha256 = "a".repeat(64);
    manifest.input_hashes.fasta_sha256 = "b".repeat(64);
    manifest.input_hashes.fai_sha256 = "c".repeat(64);
    manifest.input_hashes.policy_sha256 = "d".repeat(64);
    manifest.db_hash = "e".repeat(64);
    manifest.artifact_hash = "f".repeat(64);
    manifest.toolchain_hash = "g".repeat(64);
    manifest.created_at = "2026-02-24T00:00:00Z".to_string();
    manifest.schema_evolution_note = "additive evolution".to_string();
    manifest.ingest_toolchain = "rustc-1.80".to_string();
    manifest.ingest_build_hash = "h".repeat(64);
    manifest.qc_report_path = "derived/qc_report.json".to_string();
    manifest.source_gff3_filename = "genes.gff3.bgz".to_string();
    manifest.source_fasta_filename = "genome.fa.bgz".to_string();
    manifest.source_fai_filename = "genome.fa.bgz.fai".to_string();
    for i in 0..256 {
        manifest
            .derived_column_origins
            .insert(format!("field_{i}"), format!("origin_{i}"));
    }
    manifest
}

fn bench_manifest_encode_decode(c: &mut Criterion) {
    let manifest = large_manifest();

    c.bench_function("artifact_manifest_encode", |b| {
        b.iter(|| serde_json::to_vec(black_box(&manifest)).expect("encode"))
    });

    let encoded = serde_json::to_vec(&manifest).expect("fixture encode");
    c.bench_function("artifact_manifest_decode", |b| {
        b.iter(|| {
            let _: ArtifactManifest = serde_json::from_slice(black_box(&encoded)).expect("decode");
        })
    });
}

criterion_group!(benches, bench_manifest_encode_decode);
criterion_main!(benches);
