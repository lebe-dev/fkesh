use crate::error::FileCacheError;

pub type EmptyResult = Result<(), FileCacheError>;
pub type OptionalResult<T> = Result<Option<T>, FileCacheError>;