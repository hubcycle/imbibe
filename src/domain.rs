pub mod block;
pub mod tx;

#[derive(Debug, Clone)]
pub struct AppHash(Vec<u8>);

#[derive(Debug, Clone)]
pub struct ValidatorAddress([u8; Self::ACCOUNT_LEN]);

impl AppHash {
	pub const fn new(hash: Vec<u8>) -> Self {
		Self(hash)
	}

	pub fn get(&self) -> &[u8] {
		self.0.as_slice()
	}

	pub fn into_bytes(self) -> Vec<u8> {
		self.0
	}
}

impl ValidatorAddress {
	const ACCOUNT_LEN: usize = 20;

	pub const fn new(bytes: [u8; Self::ACCOUNT_LEN]) -> Self {
		Self(bytes)
	}

	pub const fn get(&self) -> &[u8; Self::ACCOUNT_LEN] {
		&self.0
	}
}
