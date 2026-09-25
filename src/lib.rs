#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "uuid")]
mod uuid;

use base32::Alphabet;
use core::fmt;
use rand::RngExt;
use std::time::{Duration, SystemTime};
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug, PartialEq)]
pub enum UlysError {
	ParseInvalidLength,
	ParseBase32Decode,
	ParseToArray,
}

impl fmt::Display for UlysError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		let text = match *self {
			UlysError::ParseInvalidLength => "invalid length",
			UlysError::ParseBase32Decode => "invalid character",
			UlysError::ParseToArray => "invalid array",
		};
		write!(f, "{text}")
	}
}

impl std::error::Error for UlysError {}

#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ulys(pub u128);

impl Ulys {
	/// Length of a string-encoded Ulys
	pub const ULYS_LEN: usize = 26;

	/// The number of bits in a Ulys time portion
	pub const TIME_BITS: u8 = 48;
	/// The number of bits in a Ulys random portion, including the `UUIDv8` version and variant bits
	pub const RAND_BITS: u8 = 48;
	/// The number of bits in a Ulys checksum
	pub const CHECK_BITS: u8 = 32;

	/// First valid time, 2020-01-01 00:00:00 UTC, in milliseconds since the Unix epoch
	pub const MIN_TIME_MS: u64 = 1_577_836_800_000;
	/// First time after the valid range, 2101-01-01 00:00:00 UTC, in milliseconds since the Unix epoch
	pub const MAX_TIME_MS: u64 = 4_133_980_800_000;

	/// Mask of the timestamp value, before the shift.
	const TIME_MASK: u128 = (1 << Self::TIME_BITS) - 1;
	/// Mask of the random portion.
	const RAND_MASK: u128 = ((1 << Self::RAND_BITS) - 1) << Self::CHECK_BITS;
	/// Mask of the checksum portion.
	const CHECK_MASK: u128 = (1 << Self::CHECK_BITS) - 1;
	/// Mask of the UUID version field (bits 48 to 51).
	const VERSION_MASK: u128 = 0xF << 76;
	/// UUID version 8.
	const VERSION_8: u128 = 0x8 << 76;
	/// Mask of the UUID variant field (bits 64 and 65).
	const VARIANT_MASK: u128 = 0b11 << 62;
	/// UUID variant of RFC 9562.
	const VARIANT_RFC9562: u128 = 0b10 << 62;

	/// Creates a new Ulys with the current time (UTC)
	#[must_use]
	pub fn new() -> Self {
		Self::from_datetime(SystemTime::now())
	}

	/// Creates a Ulys from a Crockford Base32 encoded string
	///
	/// # Errors
	///
	/// An `UlysError` will be returned when the given string is not formatted
	/// properly.
	pub fn from_string(s: &str) -> Result<Ulys, UlysError> {
		if s.len() != Ulys::ULYS_LEN {
			return Err(UlysError::ParseInvalidLength);
		}

		let value = base32::decode(Alphabet::Crockford, s)
			.ok_or(UlysError::ParseBase32Decode)?
			.try_into()
			.map_err(|_| UlysError::ParseToArray)?;

		Ok(Ulys(u128::from_be_bytes(value)))
	}

	/// Gets the datetime of when this Ulys was created accurate to 1ms
	#[must_use]
	pub fn datetime(&self) -> SystemTime {
		let stamp = self.timestamp_ms();
		SystemTime::UNIX_EPOCH + Duration::from_millis(stamp)
	}

	/// Checks if the checksum of the Ulys is valid
	#[must_use]
	pub fn is_valid(&self) -> bool {
		self.0 == Self::with_checksum(self.0 & !Self::CHECK_MASK)
	}

	/// Checks if the Ulys has the `UUIDv8` version and variant bits
	#[must_use]
	pub fn is_uuidv8(&self) -> bool {
		self.0 & Self::VERSION_MASK == Self::VERSION_8
			&& self.0 & Self::VARIANT_MASK == Self::VARIANT_RFC9562
	}

	/// Checks if the time of the Ulys is from 2020 to 2100, both included
	#[must_use]
	pub fn is_valid_time(&self) -> bool {
		(Self::MIN_TIME_MS..Self::MAX_TIME_MS).contains(&self.timestamp_ms())
	}

	/// Test if the Ulys is nil
	#[must_use]
	pub fn is_default(&self) -> bool {
		self.0 == 0u128
	}

	/// Creates a new Ulys with the given datetime
	fn from_datetime(datetime: SystemTime) -> Self {
		let timestamp = datetime
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or(Duration::ZERO)
			.as_millis()
			& Self::TIME_MASK;

		let random = u128::from(rand::rng().random::<u64>()) << Self::CHECK_BITS & Self::RAND_MASK;
		let random = random & !(Self::VERSION_MASK | Self::VARIANT_MASK)
			| Self::VERSION_8
			| Self::VARIANT_RFC9562;
		let data = timestamp << (Self::RAND_BITS + Self::CHECK_BITS) | random;

		Self(Self::with_checksum(data))
	}

	/// Sets the checksum bits of the given data, the checksum bits of `data` must be zero
	fn with_checksum(data: u128) -> u128 {
		data | u128::from(Self::checksum(data) >> Self::CHECK_BITS)
	}

	/// Creates a checksum for the given data
	fn checksum(data: u128) -> u64 {
		xxh3_64(data.to_be_bytes().as_slice())
	}

	/// Gets the timestamp section of this Ulys
	fn timestamp_ms(&self) -> u64 {
		(self.0 >> (Self::RAND_BITS + Self::CHECK_BITS)) as u64
	}
}

impl std::fmt::Display for Ulys {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			base32::encode(Alphabet::Crockford, &self.0.to_be_bytes()).to_lowercase()
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_default() {
		let ulys = Ulys::new();
		assert!(!ulys.is_default());

		let nil = Ulys::default();
		assert!(nil.is_default());

		assert_eq!(nil.to_string(), "00000000000000000000000000");
	}

	#[test]
	fn test_from_string() {
		let text = "068cbxpc1wy9d0v9gbhrg0020r";
		let ulys = Ulys::from_string(text);

		assert!(ulys.is_ok());

		let data = ulys.expect("failed to deserialize");
		assert_eq!(data.to_string(), text);
		assert_eq!(data.0, 2_080_933_931_387_190_948_831_204_449_898_725_894);
	}

	#[test]
	fn test_from_string_invalid_length() {
		let ulys = Ulys::from_string("ABC");

		assert!(ulys.is_err());
		assert_eq!(ulys.unwrap_err(), UlysError::ParseInvalidLength);
	}

	#[test]
	fn test_from_string_invalid_letter() {
		let ulys = Ulys::from_string("0000000000000u000000000000");

		assert!(ulys.is_err());
		assert_eq!(ulys.unwrap_err(), UlysError::ParseBase32Decode);
	}

	#[test]
	fn test_dynamic() {
		let ulys = Ulys::new();
		let encoded = ulys.to_string();
		let ulys2 = Ulys::from_string(&encoded).expect("failed to deserialize");

		assert_eq!(ulys, ulys2);
	}

	#[test]
	fn test_datetime() {
		let dt = SystemTime::now();
		let ulys = Ulys::from_datetime(dt);

		assert!(ulys.datetime() <= dt);
		assert!(ulys.datetime() + Duration::from_millis(1) >= dt);
	}

	#[test]
	fn test_timestamp() {
		let dt = SystemTime::now();
		let ulys = Ulys::from_datetime(dt);
		let ts = dt
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_millis();

		assert_eq!(u128::from(ulys.timestamp_ms()), ts);
	}

	#[test]
	fn test_order() {
		let dt = SystemTime::now();
		let ulys1 = Ulys::from_datetime(dt);
		let ulys2 = Ulys::from_datetime(dt + Duration::from_millis(1));

		assert!(ulys1 < ulys2);
	}

	#[test]
	fn test_is_valid() {
		let ulys = Ulys::from_string("068dkwmn3a441g20mzbsmyk5b8").expect("failed to deserialize");

		assert!(ulys.is_valid());
	}

	#[test]
	fn test_new_is_valid_uuidv8() {
		for _ in 0..1000 {
			let ulys = Ulys::new();
			assert!(ulys.is_valid());
			assert!(ulys.is_uuidv8());
		}
	}

	#[test]
	fn test_is_valid_time() {
		let at = |ms: u64| Ulys(u128::from(ms) << (Ulys::RAND_BITS + Ulys::CHECK_BITS));

		assert!(Ulys::new().is_valid_time());
		assert!(at(Ulys::MIN_TIME_MS).is_valid_time());
		assert!(at(Ulys::MAX_TIME_MS - 1).is_valid_time());
		assert!(!at(Ulys::MIN_TIME_MS - 1).is_valid_time());
		assert!(!at(Ulys::MAX_TIME_MS).is_valid_time());
		assert!(!Ulys::default().is_valid_time());
	}

	#[test]
	fn test_legacy_is_not_uuidv8() {
		let ulys = Ulys::from_string("068dkwmn3a441g20mzbsmyk5b8").expect("failed to deserialize");

		assert!(ulys.is_valid());
		assert!(!ulys.is_uuidv8());
	}

	#[test]
	fn test_is_not_valid() {
		let ulys = Ulys::from_string("068dkwmn3a441g20mzbsmy0000").expect("failed to deserialize");

		assert!(!ulys.is_valid());
	}
}
