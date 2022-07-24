use crate::error::FileCacheError;

pub type OperationResult<T> = Result<T, FileCacheError>;
pub type EmptyResult = Result<(), FileCacheError>;
pub type OptionalResult<T> = Result<Option<T>, FileCacheError>;