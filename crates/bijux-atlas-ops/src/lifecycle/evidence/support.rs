// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

pub fn evidence_root(repo_root: &Path) -> Result<PathBuf, String> {
    let path = repo_root.join("ops/release/evidence");
    std::fs::create_dir_all(&path)
        .map_err(|err| format!("failed to create {}: {err}", path.display()))?;
    Ok(path)
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    use sha2::{Digest, Sha256};
    Ok(hex::encode(Sha256::digest(bytes)))
}

pub fn collect_image_artifacts(repo_root: &Path) -> Result<Vec<serde_json::Value>, String> {
    let values_root = repo_root.join("ops/k8s/values");
    let mut rows = std::fs::read_dir(&values_root)
        .map_err(|err| format!("failed to read {}: {err}", values_root.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("yaml"))
        .collect::<Vec<_>>();
    rows.sort();
    let mut artifacts = Vec::new();
    for path in rows {
        let value: serde_yaml::Value = serde_yaml::from_str(
            &std::fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
        let Some(image) = value.get("image") else {
            continue;
        };
        let repository = image
            .get("repository")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let digest = image
            .get("digest")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let tag = image
            .get("tag")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if repository.is_empty() && digest.is_empty() && tag.is_empty() {
            continue;
        }
        let profile = path
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        artifacts.push(serde_json::json!({
            "source_path": path.strip_prefix(repo_root).unwrap_or(&path).display().to_string(),
            "profile": profile,
            "repository": repository,
            "digest": digest,
            "tag": tag
        }));
    }
    Ok(artifacts)
}

pub fn reset_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_dir_all(path)
            .map_err(|err| format!("failed to clear {}: {err}", path.display()))?;
    }
    std::fs::create_dir_all(path)
        .map_err(|err| format!("failed to create {}: {err}", path.display()))
}

#[must_use]
pub fn image_ref_for_evidence(row: &serde_json::Value) -> Option<String> {
    let repository = row
        .get("repository")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim();
    let digest = row
        .get("digest")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim();
    if repository.is_empty() || digest.is_empty() {
        None
    } else {
        Some(format!("{repository}@{digest}"))
    }
}

pub fn collect_sboms(
    repo_root: &Path,
    image_artifacts: &[serde_json::Value],
) -> Result<Vec<serde_json::Value>, String> {
    let evidence_root = evidence_root(repo_root)?;
    let sbom_dir = evidence_root.join("sboms");
    reset_directory(&sbom_dir)?;
    let mut rows = Vec::new();
    for image in image_artifacts {
        let profile = image
            .get("profile")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let digest = image
            .get("digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if digest.is_empty() {
            continue;
        }
        let image_ref = image_ref_for_evidence(image)
            .or_else(|| Some(digest.to_string()))
            .unwrap_or_else(|| digest.to_string());
        let sbom_path = sbom_dir.join(format!("{profile}.spdx.json"));
        let document = serde_json::json!({
            "SPDXID": "SPDXRef-DOCUMENT",
            "creationInfo": {
                "created": "1970-01-01T00:00:00Z",
                "creators": ["Tool: bijux-atlas-dev release evidence"],
                "licenseListVersion": "3.22"
            },
            "dataLicense": "CC0-1.0",
            "documentNamespace": format!("https://bijux.dev/evidence/sbom/{profile}/{digest}"),
            "name": format!("bijux-atlas {profile} image evidence"),
            "packages": [{
                "SPDXID": format!("SPDXRef-Package-{profile}"),
                "downloadLocation": "NOASSERTION",
                "externalRefs": [{
                    "referenceCategory": "PACKAGE-MANAGER",
                    "referenceLocator": image_ref,
                    "referenceType": "purl"
                }],
                "filesAnalyzed": false,
                "name": format!("bijux-atlas-{profile}"),
                "primaryPackagePurpose": "CONTAINER",
                "versionInfo": digest
            }],
            "relationships": [],
            "spdxVersion": "SPDX-2.3"
        });
        std::fs::write(
            &sbom_path,
            serde_json::to_string_pretty(&document).map_err(|err| err.to_string())?,
        )
        .map_err(|err| format!("failed to write {}: {err}", sbom_path.display()))?;
        rows.push(serde_json::json!({
            "path": sbom_path.strip_prefix(repo_root).unwrap_or(&sbom_path).display().to_string(),
            "format": "spdx-json",
            "sha256": sha256_file(&sbom_path)?,
            "image_ref": image_ref
        }));
    }
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::{
        collect_image_artifacts, collect_sboms, evidence_root, image_ref_for_evidence,
        reset_directory, sha256_file,
    };

    #[test]
    fn evidence_root_creates_release_evidence_directory() {
        let root = tempfile::tempdir().expect("tempdir");

        let path = evidence_root(root.path()).expect("evidence root");

        assert!(path.ends_with("ops/release/evidence"));
        assert!(path.exists(), "expected evidence root to exist");
    }

    #[test]
    fn sha256_file_hashes_file_contents() {
        let root = tempfile::tempdir().expect("tempdir");
        let file = root.path().join("artifact.txt");
        std::fs::write(&file, "atlas").expect("write artifact");

        let digest = sha256_file(&file).expect("sha256");

        assert_eq!(
            digest,
            "7c82602500857aa6ed0cf38c4c3e4ec645bdcaa82c00b9155eb08be100c778a9"
        );
    }

    #[test]
    fn image_ref_for_evidence_formats_repository_and_digest() {
        let row = serde_json::json!({
            "repository": "ghcr.io/bijux/atlas",
            "digest": "sha256:abc"
        });

        let image_ref = image_ref_for_evidence(&row);

        assert_eq!(image_ref.as_deref(), Some("ghcr.io/bijux/atlas@sha256:abc"));
    }

    #[test]
    fn collect_image_artifacts_reads_profile_value_files() {
        let root = tempfile::tempdir().expect("tempdir");
        let values_root = root.path().join("ops/k8s/values");
        std::fs::create_dir_all(&values_root).expect("mkdir values");
        std::fs::write(
            values_root.join("kind.yaml"),
            r#"
image:
  repository: ghcr.io/bijux/atlas
  digest: sha256:abc
  tag: latest
"#,
        )
        .expect("write kind values");

        let artifacts = collect_image_artifacts(root.path()).expect("collect image artifacts");

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0]["profile"].as_str(), Some("kind"));
    }

    #[test]
    fn collect_sboms_materializes_spdx_documents() {
        let root = tempfile::tempdir().expect("tempdir");
        let evidence_dir = evidence_root(root.path()).expect("evidence root");
        reset_directory(&evidence_dir.join("sboms")).expect("reset sboms");
        let images = vec![serde_json::json!({
            "profile": "kind",
            "repository": "ghcr.io/bijux/atlas",
            "digest": "sha256:abc"
        })];

        let rows = collect_sboms(root.path(), &images).expect("collect sboms");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["format"].as_str(), Some("spdx-json"));
        assert!(
            root.path()
                .join(rows[0]["path"].as_str().expect("sbom path"))
                .exists(),
            "expected sbom file to exist"
        );
    }
}
