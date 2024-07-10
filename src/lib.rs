#![warn(missing_docs)]
//! # ulys
//!
//! This is an adaptation of the Rust implementation of the [ulid][ulid] project which provides
//! Universally Unique Lexicographically Sortable Identifiers, with a checksum.
//!
//! [ulid]: https://github.com/ulid/spec
//!
//!
//! ## Quickstart
//!
//! ```rust
//! # use ulys::Ulys;
//! // Generate a ulys
//! # #[cfg(not(feature = "std"))]
//! # let ulys = Ulys::default();
//! # #[cfg(feature = "std")]
//! let ulys = Ulys::new();
//!
//! // Generate a string for a ulys
//! let s = ulys.to_string();
//!
//! // Create from a String
//! let res = Ulys::from_string(&s);
//! assert_eq!(ulys, res.unwrap());
//!
//! // Or using FromStr
//! let res = s.parse();
//! assert_eq!(ulys, res.unwrap());
//!
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[doc = include_str!("../README.md")]
#[cfg(all(doctest, feature = "std"))]
struct ReadMeDoctest;

mod base32;
#[cfg(feature = "std")]
mod generator;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "std")]
mod time;
#[cfg(feature = "std")]
mod time_utils;
#[cfg(feature = "uuid")]
mod uuid;

use core::fmt;
use core::str::FromStr;

pub use crate::base32::{DecodeError, EncodeError, ULYS_LEN};
#[cfg(feature = "std")]
pub use crate::generator::{Generator, MonotonicError};

/// Create a right-aligned bitmask of $len bits
macro_rules! bitmask {
    ($len:expr) => {
        ((1 << $len) - 1)
    };
}
// Allow other modules to use the macro
pub(crate) use bitmask;

/// A Ulys is a unique 128-bit lexicographically sortable identifier
///
/// Canonically, it is represented as a 26 character Crockford Base32 encoded
/// string.
///
/// Of the 128-bits, the first 48 are a unix timestamp in milliseconds. The
/// remaining 80 are random. The first 48 provide for lexicographic sorting and
/// the remaining 80 ensure that the identifier is unique.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ulys(pub u128);

impl Ulys {
    /// The number of bits in a Ulys time portion
    pub const TIME_BITS: u8 = 48;
    /// The number of bits in a Ulys random portion
    pub const RAND_BITS: u8 = 80;

    /// Create a Ulys from separated parts.
    ///
    /// NOTE: Any overflow bits in the given args are discarded
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let ulys = Ulys::from_string("01d39zy06fgsctvn4t2v9pkhfz").unwrap();
    ///
    /// let ulys2 = Ulys::from_parts(ulys.timestamp_ms(), ulys.random());
    ///
    /// assert_eq!(ulys, ulys2);
    /// ```
    pub const fn from_parts(timestamp_ms: u64, random: u128) -> Ulys {
        let time_part = (timestamp_ms & bitmask!(Self::TIME_BITS)) as u128;
        let rand_part = random & bitmask!(Self::RAND_BITS);
        Ulys((time_part << Self::RAND_BITS) | rand_part)
    }

    /// Creates a Ulys from a Crockford Base32 encoded string
    ///
    /// An `DecodeError` will be returned when the given string is not formatted
    /// properly.
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let text = "01d39zy06fgsctvn4t2v9pkhfz";
    /// let result = Ulys::from_string(text);
    ///
    /// assert!(result.is_ok());
    /// assert_eq!(&result.unwrap().to_string(), text);
    /// ```
    pub const fn from_string(encoded: &str) -> Result<Ulys, DecodeError> {
        match base32::decode(encoded) {
            Ok(int_val) => Ok(Ulys(int_val)),
            Err(err) => Err(err),
        }
    }

    /// The 'nil Ulys'.
    ///
    /// The nil Ulys is special form of Ulys that is specified to have
    /// all 128 bits set to zero.
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let ulys = Ulys::nil();
    ///
    /// assert_eq!(
    ///     ulys.to_string(),
    ///     "00000000000000000000000000"
    /// );
    /// ```
    pub const fn nil() -> Ulys {
        Ulys(0)
    }

    /// Gets the timestamp section of this ulys
    ///
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use std::time::{SystemTime, Duration};
    /// use ulys::Ulys;
    ///
    /// let dt = SystemTime::now();
    /// let ulys = Ulys::from_datetime(dt);
    ///
    /// assert_eq!(u128::from(ulys.timestamp_ms()), dt.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis());
    /// # }
    /// ```
    pub const fn timestamp_ms(&self) -> u64 {
        (self.0 >> Self::RAND_BITS) as u64
    }

    /// Gets the random section of this ulys
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let text = "01d39zy06fgsctvn4t2v9pkhfz";
    /// let ulys = Ulys::from_string(text).unwrap();
    /// let ulys_next = ulys.increment().unwrap();
    ///
    /// assert_eq!(ulys.random() + 1, ulys_next.random());
    /// ```
    pub const fn random(&self) -> u128 {
        self.0 & bitmask!(Self::RAND_BITS)
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulys
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let text = "01d39zy06fgsctvn4t2v9pkhfz";
    /// let ulys = Ulys::from_string(text).unwrap();
    ///
    /// let mut buf = [0; ulys::ULYS_LEN];
    /// let new_text = ulys.array_to_str(&mut buf);
    ///
    /// assert_eq!(new_text, text);
    /// ```
    #[deprecated(since = "1.2.0", note = "Use the infallible `array_to_str` instead.")]
    pub fn to_str<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf mut str, EncodeError> {
        #[allow(deprecated)]
        let len = base32::encode_to(self.0, buf)?;
        Ok(unsafe { core::str::from_utf8_unchecked_mut(&mut buf[..len]) })
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulys
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let text = "01d39zy06fgsctvn4t2v9pkhfz";
    /// let ulys = Ulys::from_string(text).unwrap();
    ///
    /// let mut buf = [0; ulys::ULYS_LEN];
    /// let new_text = ulys.array_to_str(&mut buf);
    ///
    /// assert_eq!(new_text, text);
    /// ```
    pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ULYS_LEN]) -> &'buf mut str {
        base32::encode_to_array(self.0, buf);
        unsafe { core::str::from_utf8_unchecked_mut(buf) }
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulys
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let text = "01d39zy06fgsctvn4t2v9pkhfz";
    /// let ulys = Ulys::from_string(text).unwrap();
    ///
    /// assert_eq!(&ulys.to_string(), text);
    /// ```
    #[allow(clippy::inherent_to_string_shadow_display)] // Significantly faster than Display::to_string
    #[cfg(feature = "std")]
    pub fn to_string(&self) -> String {
        base32::encode(self.0)
    }

    /// Test if the Ulys is nil
    ///
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// # #[cfg(not(feature = "std"))]
    /// # let ulys = Ulys(1);
    /// # #[cfg(feature = "std")]
    /// let ulys = Ulys::new();
    /// assert!(!ulys.is_nil());
    ///
    /// let nil = Ulys::nil();
    /// assert!(nil.is_nil());
    /// ```
    pub const fn is_nil(&self) -> bool {
        self.0 == 0u128
    }

    /// Increment the random number, make sure that the ts millis stays the same
    pub const fn increment(&self) -> Option<Ulys> {
        const MAX_RANDOM: u128 = bitmask!(Ulys::RAND_BITS);

        if (self.0 & MAX_RANDOM) == MAX_RANDOM {
            None
        } else {
            Some(Ulys(self.0 + 1))
        }
    }

    /// Creates a Ulys using the provided bytes array.
    ///
    /// # Example
    /// ```
    /// use ulys::Ulys;
    /// let bytes = [0xFF; 16];
    ///
    /// let ulys = Ulys::from_bytes(bytes);
    ///
    /// assert_eq!(
    ///     ulys.to_string(),
    ///     "7zzzzzzzzzzzzzzzzzzzzzzzzz"
    /// );
    /// ```
    pub const fn from_bytes(bytes: [u8; 16]) -> Ulys {
        Self(u128::from_be_bytes(bytes))
    }

    /// Returns the bytes of the Ulys in big-endian order.
    ///
    /// # Example
    /// ```
    /// use ulys::Ulys;
    ///
    /// let text = "7zzzzzzzzzzzzzzzzzzzzzzzzz";
    /// let ulys = Ulys::from_string(text).unwrap();
    ///
    /// assert_eq!(ulys.to_bytes(), [0xFF; 16]);
    /// ```
    pub const fn to_bytes(&self) -> [u8; 16] {
        self.0.to_be_bytes()
    }
}

impl Default for Ulys {
    fn default() -> Self {
        Ulys::nil()
    }
}

#[cfg(feature = "std")]
impl From<Ulys> for String {
    fn from(ulys: Ulys) -> String {
        ulys.to_string()
    }
}

impl From<(u64, u64)> for Ulys {
    fn from((msb, lsb): (u64, u64)) -> Self {
        Ulys(u128::from(msb) << 64 | u128::from(lsb))
    }
}

impl From<Ulys> for (u64, u64) {
    fn from(ulys: Ulys) -> (u64, u64) {
        ((ulys.0 >> 64) as u64, (ulys.0 & bitmask!(64)) as u64)
    }
}

impl From<u128> for Ulys {
    fn from(value: u128) -> Ulys {
        Ulys(value)
    }
}

impl From<Ulys> for u128 {
    fn from(ulys: Ulys) -> u128 {
        ulys.0
    }
}

impl From<[u8; 16]> for Ulys {
    fn from(bytes: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(bytes))
    }
}

impl From<Ulys> for [u8; 16] {
    fn from(ulys: Ulys) -> Self {
        ulys.0.to_be_bytes()
    }
}

impl FromStr for Ulys {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulys::from_string(s)
    }
}

impl fmt::Display for Ulys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut buffer = [0; ULYS_LEN];
        write!(f, "{}", self.array_to_str(&mut buffer))
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_static() {
        let s = Ulys(0x4141_4141_4141_4141_4141_4141_4141_4141).to_string();
        let u = Ulys::from_string(&s).unwrap();
        assert_eq!(&s, "21850m2ga1850m2ga1850m2ga1");
        assert_eq!(u.0, 0x4141_4141_4141_4141_4141_4141_4141_4141);
    }

    #[test]
    fn test_increment() {
        let ulys = Ulys::from_string("01bx5zzkbkazzzzzzzzzzzzzzz").unwrap();
        let ulys = ulys.increment().unwrap();
        assert_eq!("01bx5zzkbkb000000000000000", ulys.to_string());

        let ulys = Ulys::from_string("01bx5zzkbkzzzzzzzzzzzzzzzx").unwrap();
        let ulys = ulys.increment().unwrap();
        assert_eq!("01bx5zzkbkzzzzzzzzzzzzzzzy", ulys.to_string());
        let ulys = ulys.increment().unwrap();
        assert_eq!("01bx5zzkbkzzzzzzzzzzzzzzzz", ulys.to_string());
        assert!(ulys.increment().is_none());
    }

    #[test]
    fn test_increment_overflow() {
        let ulys = Ulys(u128::MAX);
        assert!(ulys.increment().is_none());
    }

    #[test]
    fn can_into_thing() {
        let ulys = Ulys::from_str("01fkmg6gag0pjanmwfn84tnxcd").unwrap();
        let s: String = ulys.into();
        let u: u128 = ulys.into();
        let uu: (u64, u64) = ulys.into();
        let bytes: [u8; 16] = ulys.into();

        assert_eq!(Ulys::from_str(&s).unwrap(), ulys);
        assert_eq!(Ulys::from(u), ulys);
        assert_eq!(Ulys::from(uu), ulys);
        assert_eq!(Ulys::from(bytes), ulys);
    }

    #[test]
    fn default_is_nil() {
        assert_eq!(Ulys::default(), Ulys::nil());
    }

    #[test]
    fn can_display_things() {
        println!("{}", Ulys::nil());
        println!("{}", EncodeError::BufferTooSmall);
        println!("{}", DecodeError::InvalidLength);
        println!("{}", DecodeError::InvalidChar);
    }
}
