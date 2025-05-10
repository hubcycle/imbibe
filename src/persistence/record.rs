pub mod insert;

use anyhow::Context;
use chrono::{DateTime, Utc};
use jiff::Timestamp;

use crate::types::Sha256;

fn bytes_to_sha256(bytes: &[u8]) -> anyhow::Result<Sha256> {
	Ok(Sha256::new(
		bytes.try_into().context("sha256 must have exactly 32 bytes")?,
	))
}

fn jiff_to_chrono(jiff: &Timestamp) -> anyhow::Result<DateTime<Utc>> {
	let nanos = jiff.as_nanosecond();

	const NANOS_IN_ONE_SEC: i128 = 1_000_000_000;

	let secs = (nanos / NANOS_IN_ONE_SEC).try_into()?;
	let sub_nanos = (nanos % NANOS_IN_ONE_SEC).try_into()?;

	DateTime::from_timestamp(secs, sub_nanos).context("failed to parse timestamp")
}

fn chrono_to_jiff(chrono: &DateTime<Utc>) -> Timestamp {
	let secs = chrono.timestamp();
	let sub_nanos = chrono.timestamp_subsec_nanos().try_into().expect("sub nanos must be valid");
	Timestamp::new(secs, sub_nanos).expect("valid datetime must yield valid timestamp")
}
