#![forbid(unsafe_code)]
//! Python packaging helpers and optional native bridge for the Bijux Atlas SDK.

/// Return the package version exposed to Python packaging workflows.
#[must_use]
pub fn package_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Return the committed compatibility JSON payload shipped with the Python SDK.
#[must_use]
pub fn compatibility_matrix_json() -> &'static str {
    include_str!("../compatibility.json")
}

/// Parse the committed compatibility JSON payload.
pub fn compatibility_matrix() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(compatibility_matrix_json())
}

#[cfg(feature = "python-extension")]
mod python_extension {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    use crate::{compatibility_matrix, compatibility_matrix_json, package_version};

    #[pyfunction]
    fn version() -> &'static str {
        package_version()
    }

    #[pyfunction]
    fn compatibility_json() -> &'static str {
        compatibility_matrix_json()
    }

    #[pyfunction]
    fn compatibility() -> PyResult<String> {
        compatibility_matrix()
            .and_then(|value| serde_json::to_string(&value))
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[pymodule]
    fn _native(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_function(wrap_pyfunction!(version, module)?)?;
        module.add_function(wrap_pyfunction!(compatibility_json, module)?)?;
        module.add_function(wrap_pyfunction!(compatibility, module)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{compatibility_matrix, package_version};

    #[test]
    fn package_version_matches_workspace_version() {
        assert_eq!(package_version(), "0.1.1");
    }

    #[test]
    fn compatibility_json_is_valid() {
        let parsed = compatibility_matrix().expect("compatibility json should parse");
        assert_eq!(parsed["schema_version"], 1);
        assert!(parsed.get("client_major_support").is_some());
    }
}
