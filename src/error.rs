use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileCacheError {
    #[error("File cache error")]
    Default,

    #[error(transparent)]
    EncodingError(#[from] serde_json::Error),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}