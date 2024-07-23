#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "uuid")]
mod uuid;

use base32::Alphabet;
use core::fmt;
use rand::Rng;
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

#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ulys(pub u128);

impl Ulys {
    /// Length of a string-encoded Ulys
    pub const ULYS_LEN: usize = 26;

    /// The number of bits in a Ulys time portion
    pub const TIME_BITS: u8 = 48;
    /// The number of bits in a Ulys random portion
    pub const RAND_BITS: u8 = 48;
    /// The number of bits in a Ulys cheksum
    pub const CHECK_BITS: u8 = 32;

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

    /// Checks if the Ulys is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let data = (self.0 >> Self::CHECK_BITS) << Self::CHECK_BITS;
        let checksum = Ulys::checksum(data);

        self.0 == (data | u128::from(checksum >> Self::CHECK_BITS))
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
            .as_millis();

        let mut source = rand::thread_rng();
        let msb = timestamp << (64 - Self::TIME_BITS) | u128::from(u64::from(source.gen::<u16>()));
        let rand = source.gen::<u64>();
        let data = msb << 64 | u128::from(rand << 32);
        let checksum = Ulys::checksum(data);
        let lsb = (rand << Self::CHECK_BITS) | checksum >> Self::CHECK_BITS;

        Self(msb << 64 | u128::from(lsb))
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
    fn test_is_not_valid() {
        let ulys = Ulys::from_string("068dkwmn3a441g20mzbsmy0000").expect("failed to deserialize");

        assert!(!ulys.is_valid());
    }
}
