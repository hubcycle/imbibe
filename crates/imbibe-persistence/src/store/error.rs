use std::borrow::Cow;

use crate::record::error::InvalidValueError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
	#[error("db error: {0}")]
	Db(#[from] diesel::result::Error),

	#[error("invalid value error: {0}")]
	InvalidValue(#[from] InvalidValueError),

	#[error("arithmetic error")]
	Arithmetic,

	#[error("other error: {0}")]
	Other(Cow<'static, str>),
}
