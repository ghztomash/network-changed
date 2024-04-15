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
    #[error("Utf8 error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Time error")]
    TimeError(#[from] std::time::SystemTimeError),
    #[cfg(feature = "encryption")]
    #[error("Encryption error")]
    EncryptionError(String),
    #[cfg(feature = "encryption")]
    #[error("Machine identifier error")]
    IdentifierError(String),
}
