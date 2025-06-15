use imbibe_persistence::{pool::PoolError, store::StoreError};

pub type Result<T, E = IndexerError> = core::result::Result<T, E>;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
	#[error("rpc error: {0}")]
	Rpc(#[from] tendermint_rpc::Error),

	#[error("block data error: {0}")]
	BlockData(String),

	#[error("timestamp error: {0}")]
	Timestamp(#[from] jiff::Error),

	#[error("height error: height must be positive")]
	Height,

	#[error("gas error: gas must not exceed {}", u64::MAX)]
	Gas,

	#[error("block hash error: block hash must be present")]
	BlockHash,

	#[error("validator hash error: validator hash must be present")]
	ValidatorHash,

	#[error("next validators hash error: next validators hash must be present")]
	NextValidatorsHash,

	#[error("consensus hash error: consensus hash must be present")]
	ConsensusHash,

	#[error("tx decode error: valid must be decodable")]
	TxDecodeError,

	#[error("bech32 address error: invalid bech32 address: {0}")]
	Bech32Address(String),

	#[error("address error: address must be exactly 20 bytes long")]
	Address,

	#[error("signer error: {0}")]
	Signer(String),

	#[error("tx msgs missing error: tx must contain at least one msg")]
	TxMsgsMissing,

	#[error("unsupported public key error: unsupported public key type")]
	UnsupportedPublicKey,

	#[error(
		"txs in block error: number of txs in single block must not exceed {}",
		u64::MAX
	)]
	TxsInBlock,

	#[error("store error: {0}")]
	Store(#[from] StoreError),

	#[error("db pool error: {0}")]
	DbPool(#[from] PoolError),

	#[error(
		"rpc height error: tendermint rpc only accepts height upto {}",
		i64::MAX
	)]
	RpcHeight,

	#[error("other error: {0}")]
	Other(Box<dyn std::error::Error + Send + Sync>),
}
