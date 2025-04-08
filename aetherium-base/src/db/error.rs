use std::{io, path::PathBuf};

use aetherium_core::{ChainCommunicationError, AetheriumProtocolError};

/// DB Error type
#[derive(thiserror::Error, Debug)]
pub enum DbError {
    /// Rocks DB Error
    #[error("{0}")]
    RockError(#[from] rocksdb::Error),
    #[error("Failed to open {path}, canonicalized as {canonicalized}: {source}")]
    /// Error opening the database
    OpeningError {
        /// Rocksdb error during opening
        #[source]
        source: Box<rocksdb::Error>,
        /// Raw database path provided
        path: PathBuf,
        /// Parsed path used
        canonicalized: PathBuf,
    },
    /// Could not parse the provided database path string
    #[error("Invalid database path supplied {1:?}; {0}")]
    InvalidDbPath(#[source] io::Error, String),
    /// Aetherium Error
    #[error("{0}")]
    AetheriumError(#[from] AetheriumProtocolError),
}

impl From<DbError> for ChainCommunicationError {
    fn from(value: DbError) -> Self {
        ChainCommunicationError::from_other(value)
    }
}
