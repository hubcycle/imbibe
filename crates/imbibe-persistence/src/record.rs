pub mod error;
pub mod insert;
pub mod select;

use chrono::{DateTime, Utc};
use jiff::Timestamp;

fn jiff_to_chrono(jiff: &Timestamp) -> Option<DateTime<Utc>> {
	let nanos = jiff.as_nanosecond();

	const NANOS_IN_ONE_SEC: i128 = 1_000_000_000;

	let secs = (nanos / NANOS_IN_ONE_SEC).try_into().ok()?;
	let sub_nanos = (nanos % NANOS_IN_ONE_SEC).try_into().ok()?;

	DateTime::from_timestamp(secs, sub_nanos)
}

fn chrono_to_jiff(chrono: &DateTime<Utc>) -> Timestamp {
	let secs = chrono.timestamp();
	let sub_nanos = chrono.timestamp_subsec_nanos().try_into().expect("sub nanos must be valid");
	Timestamp::new(secs, sub_nanos).expect("valid datetime must yield valid timestamp")
}
