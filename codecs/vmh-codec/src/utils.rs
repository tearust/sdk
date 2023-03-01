use crate::{error::Result, io::HostType};
use chrono::NaiveDateTime;
use std::{
	convert::TryInto,
	time::{Duration, SystemTime},
};
use tapp_common::TimestampShort;
use tea_sdk::{defs::FreezeTimeSettings, OptionExt};

pub use chrono::{offset::Utc, DateTime, NaiveTime};

pub fn split_once<'a>(in_string: &'a str, pattern: &'a str) -> Result<(&'a str, &'a str)> {
	let mut splitter = in_string.splitn(2, pattern);
	let first = splitter
		.next()
		.ok_or_else(|| "missing first section".to_string())?;
	let second = splitter
		.next()
		.ok_or_else(|| "missing second section".to_string())?;
	Ok((first, second))
}

pub fn local_host_url(port: u32) -> String {
	format!("0.0.0.0:{port}")
}

pub fn remote_url(host: &HostType, port: u32) -> String {
	format!("{host}:{port}")
}

pub fn system_time_as_nanos(time: SystemTime) -> Result<u128> {
	Ok(time
		.duration_since(std::time::SystemTime::UNIX_EPOCH)?
		.as_nanos())
}

pub fn system_time_from_nanos(nanos: u128) -> Result<SystemTime> {
	const NANOS_PER_SEC: u128 = 1_000_000_000;
	let sub_nanos: u32 = (nanos % NANOS_PER_SEC).try_into()?;
	let seconds: u64 = (nanos / NANOS_PER_SEC).try_into()?;

	let duration = Duration::new(seconds, sub_nanos);
	Ok(std::time::SystemTime::UNIX_EPOCH
		.checked_add(duration)
		.ok_or(format!("calculate system time from nanos {nanos} failed"))?)
}

pub fn to_short_timestamp(ts: u128) -> Result<TimestampShort> {
	let time = DateTime::<Utc>::from(system_time_from_nanos(ts)?);
	Ok(time.timestamp())
}

pub fn to_full_timestamp(ts: TimestampShort) -> Result<u128> {
	let utc = datetime_from_timestamp(ts).ok_or_err("utc time")?;
	system_time_as_nanos(utc.into())
}

pub fn datetime_from_timestamp(ts: TimestampShort) -> Option<DateTime<Utc>> {
	let local = NaiveDateTime::from_timestamp_opt(ts, 0)?;
	Some(DateTime::<Utc>::from_local(local, Utc))
}

pub fn format_timestamp(ts: TimestampShort) -> Option<String> {
	let utc = datetime_from_timestamp(ts)?;
	Some(utc.to_string())
}

pub fn format_system_time(time: SystemTime) -> String {
	let datetime: DateTime<Utc> = time.into();
	format!("{}", datetime.format("%d/%m/%Y %T"))
}

pub fn should_freeze(time: &FreezeTimeSettings) -> bool {
	if time.schedule_at == 0 || time.schedule_at < time.freeze_before as i64 {
		return false;
	}

	match datetime_from_timestamp(time.schedule_at) {
		Some(schedule_at) => {
			let now = Utc::now();
			now > schedule_at - chrono::Duration::seconds(time.freeze_before as i64)
				&& now < schedule_at + chrono::Duration::seconds(time.freeze_after as i64)
		}
		None => false,
	}
}

#[cfg(test)]
mod tests {
	use chrono::{DateTime, Utc};

	use super::{format_timestamp, split_once, to_short_timestamp};
	use crate::error::Result;

	#[test]
	fn split_works() -> Result<()> {
		let (first, second) = split_once("abc:123", ":")?;
		assert_eq!(first, "abc");
		assert_eq!(second, "123");
		Ok(())
	}

	#[test]
	fn multi_splitters_works() -> Result<()> {
		let (first, second) = split_once("abc:123:789", ":")?;
		assert_eq!(first, "abc");
		assert_eq!(second, "123:789");
		Ok(())
	}

	#[test]
	fn split_empty() -> Result<()> {
		let rtn = split_once("", ":");
		rtn.unwrap_err();

		let rtn = split_once("abc", ":");
		rtn.unwrap_err();

		Ok(())
	}

	#[test]
	fn partial_empty_is_valid() -> Result<()> {
		let (first, second) = split_once("abc:", ":")?;
		assert_eq!(first, "abc");
		assert_eq!(second, "");

		let (first, second) = split_once(":abc", ":")?;
		assert_eq!(first, "");
		assert_eq!(second, "abc");

		Ok(())
	}

	#[test]
	fn to_short_timestamp_works() -> Result<()> {
		let short = to_short_timestamp(1673816999950360000u128)?;
		let f = format_timestamp(short).ok_or("format")?;
		assert_eq!("2023-01-15 21:09:59 UTC", f);

		let utc: DateTime<Utc> = f.parse()?;
		assert_eq!(short, utc.timestamp());

		Ok(())
	}
}
