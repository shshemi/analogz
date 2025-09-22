#[derive(Debug, thiserror::Error)]
#[error("Datetime not found")]
pub struct InvalidIndexError(pub usize);
