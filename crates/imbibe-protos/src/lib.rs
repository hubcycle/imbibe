#[cfg(feature = "protogen")]
mod error;

pub use imbibe_macros::GetSigners;

#[cfg(feature = "protogen")]
pub use self::error::ProtosError;

#[allow(unused_macros)]
#[cfg(feature = "protogen")]
macro_rules! generate_signer_extractors {
	(
		$(($type_url:literal, $rust_struct:path)),* $(,)?
	) => {
		pub fn signers_from_any_msg(
			msg: &::cosmrs::Any,
		) -> Result<Box<dyn Iterator<Item = String>>, error::ProtosError> {
			match msg.type_url.as_str() {
				$(
					$type_url => {
						::cosmrs::Any::to_msg::<$rust_struct>(msg)
							.map(GetSigners::signers)
							.map(|signers| Box::new(signers) as Box<dyn Iterator<Item = String>>)
							.map_err(From::from)
					},
				)*
				_ => Err(error::ProtosError::NoSignerInMsg { type_url: msg.type_url.clone() }),
			}
		}

		#[allow(unused_variables)]
		pub fn extend_with_signers_from_any_msg<E>(
			msg: &::cosmrs::Any,
			signers: &mut E,
		) -> Result<(), error::ProtosError>
		where
			E: ::std::iter::Extend<String>,
		{
			match msg.type_url.as_str() {
				$(
					$type_url => {
						::cosmrs::Any::to_msg::<$rust_struct>(msg)
							.map(GetSigners::signers)
							.map(|s| ::std::iter::Extend::extend(signers, s))
							.map_err(From::from)
					},
				)*
				_ => Err(error::ProtosError::NoSignerInMsg { type_url: msg.type_url.clone() }),
			}
		}

		pub fn unique_signers_from_any_msg(
			msg: &::cosmrs::Any,
		) -> Result<::std::collections::HashSet<String>, error::ProtosError>
		{
			let mut unique_signers = std::collections::HashSet::new();
			extend_with_signers_from_any_msg(msg, &mut unique_signers)?;

			Ok(unique_signers)
		}

		pub fn unique_signers_from_any_msgs<'a, I>(
			msgs: I,
		) -> Result<::std::collections::HashSet<String>, error::ProtosError>
		where
			I: IntoIterator<Item = &'a cosmrs::Any> + 'a,
		{
			let mut unique_signers = std::collections::HashSet::new();
			msgs.into_iter()
				.map(|msg| extend_with_signers_from_any_msg(msg, &mut unique_signers))
				.collect::<Result<(), _>>()?;

			Ok(unique_signers)
		}
	};
}

#[cfg(feature = "protogen")]
include!(concat!(env!("OUT_DIR"), "/any_signer_extractor.rs"));

#[cfg(feature = "protogen")]
include!(concat!(env!("OUT_DIR"), "/code_gen/mod.rs"));

pub trait GetSigners {
	type Signer;

	fn signers(self) -> impl Iterator<Item = Self::Signer>;
}
