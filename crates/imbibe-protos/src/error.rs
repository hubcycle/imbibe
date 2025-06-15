use prost::DecodeError;

#[derive(Debug, thiserror::Error)]
pub enum ProtosError {
	#[error("decode error: {0}")]
	Decode(#[from] DecodeError),

	#[error("no signer in msg error: msg type url '{type_url}'")]
	NoSignerInMsg { type_url: String },
}
