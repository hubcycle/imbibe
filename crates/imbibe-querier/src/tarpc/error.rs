use serde::{Deserialize, Serialize};

use crate::QuerierError;

pub type Result<T, E = QueryTarpcError> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[error("{msg}")]
pub struct QueryTarpcError {
	msg: String,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum QueryTarpcErrorKind {
	#[error("timeout error: {0}")]
	Timeout(#[from] tokio::time::error::Elapsed),

	#[error("querier error: {0}")]
	Querier(#[from] QuerierError),
}

impl From<QueryTarpcErrorKind> for QueryTarpcError {
	fn from(err: QueryTarpcErrorKind) -> Self {
		Self { msg: err.to_string() }
	}
}
