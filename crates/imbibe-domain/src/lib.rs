pub mod block;
pub mod tx;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use cosmrs::tendermint::account::Id;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sha256([u8; Self::LEN]);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Address(Id);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NonEmptyBz<T>(T);

impl Sha256 {
	pub const LEN: usize = 32;

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

impl Address {
	const ACCOUNT_LEN: usize = 20;

	pub fn new(bz: [u8; Self::ACCOUNT_LEN]) -> Self {
		Self(Id::new(bz))
	}

	pub fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl<T> NonEmptyBz<T>
where
	T: AsRef<[u8]>,
{
	pub fn new(bz: T) -> Option<Self> {
		(!bz.as_ref().is_empty()).then_some(bz).map(Self)
	}
}

impl<T> NonEmptyBz<T> {
	pub fn get(&self) -> &T {
		&self.0
	}

	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> AsRef<[u8]> for NonEmptyBz<T>
where
	T: AsRef<[u8]>,
{
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

