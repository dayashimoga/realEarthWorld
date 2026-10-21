use thiserror::Error;

#[derive(Error, Debug)]
pub enum RealEarthError {
    #[error("Geospatial error: {0}")]
    Geospatial(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Ingestion error: {0}")]
    Ingestion(String),

    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("Streaming error: {0}")]
    Streaming(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, RealEarthError>;
