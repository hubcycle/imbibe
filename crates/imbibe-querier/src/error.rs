use imbibe_persistence::{pool::PoolError, store::StoreError};

pub type Result<T, E = QuerierError> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum QuerierError {
	#[error("store error: {0}")]
	Store(#[from] StoreError),

	#[error("db pool error: {0}")]
	DbPool(#[from] PoolError),
}

