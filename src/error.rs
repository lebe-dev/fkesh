use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileCacheError {
    #[error("File cache error")]
    Default,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}