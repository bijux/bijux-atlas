// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Local,
    HttpReadonly,
    S3Like,
}

impl BackendKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::HttpReadonly => "http-readonly",
            Self::S3Like => "s3-like",
        }
    }
}

pub fn validate_backend_compiled(kind: BackendKind) -> Result<(), String> {
    match kind {
        BackendKind::Local => Ok(()),
        BackendKind::HttpReadonly | BackendKind::S3Like => {
            #[cfg(feature = "backend-s3")]
            {
                Ok(())
            }
            #[cfg(not(feature = "backend-s3"))]
            {
                Err(format!(
                    "backend `{}` is not compiled in; rebuild with `--features backend-s3`",
                    kind.as_str()
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_backend_compiled, BackendKind};

    #[test]
    fn local_backend_is_always_compiled() {
        let result = validate_backend_compiled(BackendKind::Local);
        assert!(result.is_ok(), "local backend must always be available");
    }
}
