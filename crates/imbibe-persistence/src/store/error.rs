use crate::record::error::InvalidValueError;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
	#[error("db error: {0}")]
	Db(#[from] diesel::result::Error),

	#[error("invalid value error: {0}")]
	InvalidValue(#[from] InvalidValueError),
}
