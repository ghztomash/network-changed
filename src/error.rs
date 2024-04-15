//! # ❌ Crate errors

use thiserror::Error;

/// Result type wrapper for the crate
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the crate
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("File error")]
    FileError(#[from] std::io::Error),
    #[error("Encryption error")]
    EncryptionError(String),
}
