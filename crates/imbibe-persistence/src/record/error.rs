use core::{array::TryFromSliceError, num::TryFromIntError};

use cosmrs::ErrorReport;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum InvalidValueError {
	#[error(transparent)]
	IntConversionError(#[from] TryFromIntError),

	#[error("amount error: amount must be valid u128")]
	AmountError,

	#[error("cosmrs error: {0}")]
	Cosmrs(#[from] ErrorReport),

	#[error("empty error: value must be non empty")]
	Empty,

	#[error("slice error: {0}")]
	Slice(#[from] TryFromSliceError),

	#[error("invalid time value")]
	Time,

	#[error("json error: {0}")]
	Json(#[from] serde_json::Error),

	#[error("other error: {0}")]
	Other(String),
}
