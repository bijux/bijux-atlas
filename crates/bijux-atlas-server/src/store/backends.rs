use crate::{CacheError, CatalogFetch, DatasetStoreBackend};
use async_trait::async_trait;
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, ETAG, IF_NONE_MATCH, RANGE};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_backoff_ms: 120,
        }
    }
}

pub struct LocalFsBackend {
    root: PathBuf,
}

impl LocalFsBackend {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[async_trait]
impl DatasetStoreBackend for LocalFsBackend {
    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError> {
        let path = self.root.join("catalog.json");
        let bytes = fs::read(&path).map_err(|e| CacheError(format!("catalog read failed: {e}")))?;
        let etag = sha256_hex(&bytes);
        if if_none_match == Some(etag.as_str()) {
            return Ok(CatalogFetch::NotModified);
        }
        let catalog: Catalog = serde_json::from_slice(&bytes)
            .map_err(|e| CacheError(format!("catalog parse failed: {e}")))?;
        Ok(CatalogFetch::Updated { etag, catalog })
    }

    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        let path = artifact_paths(Path::new(&self.root), dataset).manifest;
        let bytes = fs::read(path).map_err(|e| CacheError(format!("manifest read failed: {e}")))?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|e| CacheError(format!("manifest parse failed: {e}")))?;
        manifest
            .validate_strict()
            .map_err(|e| CacheError(format!("manifest validation failed: {e}")))?;
        Ok(manifest)
    }

    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let path = artifact_paths(Path::new(&self.root), dataset).sqlite;
        fs::read(path).map_err(|e| CacheError(format!("sqlite read failed: {e}")))
    }

    async fn fetch_fasta_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let path = artifact_paths(Path::new(&self.root), dataset).fasta;
        fs::read(path).map_err(|e| CacheError(format!("fasta read failed: {e}")))
    }

    async fn fetch_fai_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let path = artifact_paths(Path::new(&self.root), dataset).fai;
        fs::read(path).map_err(|e| CacheError(format!("fai read failed: {e}")))
    }

    async fn fetch_release_gene_index_bytes(
        &self,
        dataset: &DatasetId,
    ) -> Result<Vec<u8>, CacheError> {
        let path = artifact_paths(Path::new(&self.root), dataset).release_gene_index;
        fs::read(path).map_err(|e| CacheError(format!("release gene index read failed: {e}")))
    }
}

pub struct S3LikeBackend {
    base_url: String,
    presigned_base_url: Option<String>,
    auth_bearer: Option<String>,
    retry: RetryPolicy,
}

impl S3LikeBackend {
    #[must_use]
    pub fn new(
        base_url: String,
        presigned_base_url: Option<String>,
        auth_bearer: Option<String>,
        retry: RetryPolicy,
    ) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            presigned_base_url: presigned_base_url
                .map(|x| x.trim_end_matches('/').to_string())
                .filter(|x| !x.is_empty()),
            auth_bearer,
            retry,
        }
    }

    fn object_url(&self, dataset: &DatasetId, file: &str) -> String {
        let base = self.presigned_base_url.as_deref().unwrap_or(&self.base_url);
        format!(
            "{}/{}/{}/{}/derived/{}",
            base, dataset.release, dataset.species, dataset.assembly, file
        )
    }

    fn object_url_input(&self, dataset: &DatasetId, file: &str) -> String {
        let base = self.presigned_base_url.as_deref().unwrap_or(&self.base_url);
        format!(
            "{}/{}/{}/{}/inputs/{}",
            base, dataset.release, dataset.species, dataset.assembly, file
        )
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("reqwest client build")
    }

    fn auth_headers(&self) -> Result<HeaderMap, CacheError> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.auth_bearer {
            let value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| CacheError(format!("invalid auth header: {e}")))?;
            headers.insert(AUTHORIZATION, value);
        }
        Ok(headers)
    }

    async fn get_with_retry(&self, url: &str) -> Result<Vec<u8>, CacheError> {
        let client = self.client();
        let headers = self.auth_headers()?;
        let mut attempt = 0;
        loop {
            attempt += 1;
            let req = client.get(url).headers(headers.clone());
            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    return resp
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(|e| CacheError(format!("read body failed: {e}")));
                }
                Ok(resp) => {
                    if attempt >= self.retry.max_attempts {
                        return Err(CacheError(format!(
                            "download failed status={} url={url}",
                            resp.status()
                        )));
                    }
                }
                Err(e) => {
                    if attempt >= self.retry.max_attempts {
                        return Err(CacheError(format!("download failed url={url}: {e}")));
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(
                self.retry.base_backoff_ms.saturating_mul(attempt as u64),
            ))
            .await;
        }
    }

    async fn get_catalog_with_retry(
        &self,
        url: &str,
        if_none_match: Option<&str>,
    ) -> Result<CatalogFetch, CacheError> {
        let client = self.client();
        let base_headers = self.auth_headers()?;
        let mut attempt = 0;
        loop {
            attempt += 1;
            let mut headers = base_headers.clone();
            if let Some(tag) = if_none_match {
                headers.insert(
                    IF_NONE_MATCH,
                    HeaderValue::from_str(tag)
                        .map_err(|e| CacheError(format!("invalid if-none-match header: {e}")))?,
                );
            }
            let req = client.get(url).headers(headers);
            match req.send().await {
                Ok(resp) if resp.status().as_u16() == 304 => return Ok(CatalogFetch::NotModified),
                Ok(resp) if resp.status().is_success() => {
                    let header_etag = resp
                        .headers()
                        .get(ETAG)
                        .and_then(|v| v.to_str().ok())
                        .map(ToString::to_string);
                    let bytes = resp
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(|e| CacheError(format!("read body failed: {e}")))?;
                    let catalog: Catalog = serde_json::from_slice(&bytes)
                        .map_err(|e| CacheError(format!("catalog parse failed: {e}")))?;
                    let etag = header_etag.unwrap_or_else(|| sha256_hex(&bytes));
                    return Ok(CatalogFetch::Updated { etag, catalog });
                }
                Ok(resp) => {
                    if attempt >= self.retry.max_attempts {
                        return Err(CacheError(format!(
                            "download failed status={} url={url}",
                            resp.status()
                        )));
                    }
                }
                Err(e) => {
                    if attempt >= self.retry.max_attempts {
                        return Err(CacheError(format!("download failed url={url}: {e}")));
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(
                self.retry.base_backoff_ms.saturating_mul(attempt as u64),
            ))
            .await;
        }
    }

    async fn get_resume_with_retry(&self, url: &str) -> Result<Vec<u8>, CacheError> {
        let client = self.client();
        let base_headers = self.auth_headers()?;
        let mut attempt = 0;
        let mut buf: Vec<u8> = Vec::new();
        loop {
            attempt += 1;
            let mut headers = base_headers.clone();
            if !buf.is_empty() {
                let range = format!("bytes={}-", buf.len());
                headers.insert(
                    RANGE,
                    HeaderValue::from_str(&range)
                        .map_err(|e| CacheError(format!("invalid range header: {e}")))?,
                );
            }
            let req = client.get(url).headers(headers);
            match req.send().await {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 206 => {
                    let status = resp.status();
                    let content_range = resp
                        .headers()
                        .get("content-range")
                        .and_then(|v| v.to_str().ok())
                        .map(ToString::to_string);
                    let mut part = resp
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(|e| CacheError(format!("read body failed: {e}")))?;
                    if part.is_empty() {
                        return Ok(buf);
                    }
                    buf.append(&mut part);
                    if let Some(total) = content_range
                        .as_deref()
                        .and_then(|v| v.split('/').nth(1))
                        .and_then(|v| v.parse::<usize>().ok())
                    {
                        if buf.len() >= total {
                            return Ok(buf);
                        }
                    }
                    if status.as_u16() == 206 && attempt < self.retry.max_attempts {
                        tokio::time::sleep(Duration::from_millis(
                            self.retry.base_backoff_ms.saturating_mul(attempt as u64),
                        ))
                        .await;
                        continue;
                    }
                    return Ok(buf);
                }
                Ok(resp) => {
                    if attempt >= self.retry.max_attempts {
                        return Err(CacheError(format!(
                            "resumable download failed status={} url={url}",
                            resp.status()
                        )));
                    }
                }
                Err(e) => {
                    if attempt >= self.retry.max_attempts {
                        return Err(CacheError(format!(
                            "resumable download failed url={url}: {e}"
                        )));
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(
                self.retry.base_backoff_ms.saturating_mul(attempt as u64),
            ))
            .await;
        }
    }
}

#[async_trait]
impl DatasetStoreBackend for S3LikeBackend {
    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError> {
        let url = format!("{}/catalog.json", self.base_url);
        self.get_catalog_with_retry(&url, if_none_match).await
    }

    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        let url = self.object_url(dataset, "manifest.json");
        let bytes = self.get_with_retry(&url).await?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|e| CacheError(format!("manifest parse failed: {e}")))?;
        manifest
            .validate_strict()
            .map_err(|e| CacheError(format!("manifest validation failed: {e}")))?;
        Ok(manifest)
    }

    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let url = self.object_url(dataset, "gene_summary.sqlite");
        self.get_resume_with_retry(&url).await
    }

    async fn fetch_fasta_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let url = self.object_url_input(dataset, "genome.fa.bgz");
        self.get_resume_with_retry(&url).await
    }

    async fn fetch_fai_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let url = self.object_url_input(dataset, "genome.fa.bgz.fai");
        self.get_with_retry(&url).await
    }

    async fn fetch_release_gene_index_bytes(
        &self,
        dataset: &DatasetId,
    ) -> Result<Vec<u8>, CacheError> {
        let url = self.object_url(dataset, "release_gene_index.json");
        self.get_with_retry(&url).await
    }
}
