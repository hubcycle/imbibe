#[derive(Debug, Clone)]
pub struct Sha256([u8; Self::LEN]);

impl Sha256 {
	const LEN: usize = 32;

	pub const fn new(hash: [u8; Self::LEN]) -> Self {
		Self(hash)
	}

	pub const fn get(&self) -> &[u8; Self::LEN] {
		&self.0
	}
}

impl<T> From<T> for Sha256
where
	[u8; Self::LEN]: From<T>,
{
	fn from(hash: T) -> Self {
		Self::new(hash.into())
	}
}
