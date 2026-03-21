use std::fmt;

/// Errors that can occur when building or rendering charts.
#[derive(Debug)]
pub enum ChartError {
    /// A required field was not set on a builder.
    MissingField(&'static str),
    /// DataFrame serialization failed.
    Serialization(polars::error::PolarsError),
    /// Python execution failed.
    Python(pyo3::PyErr),
    /// The embedded Python script contains a null byte.
    InvalidScript,
}

impl fmt::Display for ChartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChartError::MissingField(field) => write!(f, "missing required field: {field}"),
            ChartError::Serialization(e) => write!(f, "DataFrame serialization failed: {e}"),
            ChartError::Python(e) => write!(f, "Python execution failed: {e}"),
            ChartError::InvalidScript => write!(f, "embedded Python script contains a null byte"),
        }
    }
}

impl std::error::Error for ChartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ChartError::Serialization(e) => Some(e),
            ChartError::Python(e) => Some(e),
            _ => None,
        }
    }
}

impl From<polars::error::PolarsError> for ChartError {
    fn from(e: polars::error::PolarsError) -> Self {
        ChartError::Serialization(e)
    }
}

impl From<pyo3::PyErr> for ChartError {
    fn from(e: pyo3::PyErr) -> Self {
        ChartError::Python(e)
    }
}
