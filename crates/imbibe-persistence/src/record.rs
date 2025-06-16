pub mod error;

#[cfg(feature = "database")]
pub mod insert;

#[cfg(feature = "database")]
pub mod select;

#[cfg(feature = "database")]
fn jiff_to_chrono(jiff: &jiff::Timestamp) -> Option<chrono::DateTime<chrono::Utc>> {
	let nanos = jiff.as_nanosecond();

	const NANOS_IN_ONE_SEC: i128 = 1_000_000_000;

	let secs = (nanos / NANOS_IN_ONE_SEC).try_into().ok()?;
	let sub_nanos = (nanos % NANOS_IN_ONE_SEC).try_into().ok()?;

	chrono::DateTime::from_timestamp(secs, sub_nanos)
}

#[cfg(feature = "database")]
fn chrono_to_jiff(chrono: &chrono::DateTime<chrono::Utc>) -> jiff::Timestamp {
	let secs = chrono.timestamp();
	let sub_nanos = chrono.timestamp_subsec_nanos().try_into().expect("sub nanos must be valid");
	jiff::Timestamp::new(secs, sub_nanos).expect("valid datetime must yield valid timestamp")
}
