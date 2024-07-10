use core::fmt;

/// Length of a string-encoded Ulys
pub const ULYS_LEN: usize = 26;

const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

const NO_VALUE: u8 = 255;
const LOOKUP: [u8; 256] = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 10, 11,
    12, 13, 14, 15, 16, 17, 255, 18, 19, 255, 20, 21, 255, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30,
    31, 255, 255, 255, 255, 255, 255, 10, 11, 12, 13, 14, 15, 16, 17, 255, 18, 19, 255, 20, 21,
    255, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

/// Generator code for `LOOKUP`
#[cfg(test)]
#[test]
fn test_lookup_table() {
    let mut lookup = [NO_VALUE; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        lookup[c as usize] = u8::try_from(i).unwrap();
        if !(c as char).is_numeric() {
            //lowercase
            lookup[(c + 32) as usize] = u8::try_from(i).unwrap();
        }
    }
    assert_eq!(LOOKUP, lookup);
}

/// An error that can occur when encoding a base32 string
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum EncodeError {
    /// The length of the provided buffer is not large enough
    BufferTooSmall,
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            EncodeError::BufferTooSmall => "buffer too small",
        };
        write!(f, "{text}")
    }
}

/// Encode a u128 value to a given buffer. The provided buffer should be at least `ULYS_LEN` long.
#[deprecated(
    since = "1.2.0",
    note = "Use the infallible `encode_to_array` instead."
)]
pub fn encode_to(mut value: u128, buffer: &mut [u8]) -> Result<usize, EncodeError> {
    // NOTE: This function can't be made const because mut refs aren't allowed for some reason

    if buffer.len() < ULYS_LEN {
        return Err(EncodeError::BufferTooSmall);
    }

    for i in 0..ULYS_LEN {
        buffer[ULYS_LEN - 1 - i] = ALPHABET[(value & 0x1f) as usize];
        value >>= 5;
    }

    Ok(ULYS_LEN)
}

/// Encode a u128 value to a given buffer.
pub fn encode_to_array(mut value: u128, buffer: &mut [u8; ULYS_LEN]) {
    // NOTE: This function can't be made const because mut refs aren't allowed for some reason

    for i in 0..ULYS_LEN {
        buffer[ULYS_LEN - 1 - i] = ALPHABET[(value & 0x1f) as usize];
        value >>= 5;
    }
}

#[cfg(feature = "std")]
pub fn encode(value: u128) -> String {
    let mut buffer: [u8; ULYS_LEN] = [0; ULYS_LEN];

    encode_to_array(value, &mut buffer);

    String::from_utf8(buffer.to_vec()).expect("unexpected failure in base32 encode for ulys")
}

/// An error that can occur when decoding a base32 string
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum DecodeError {
    /// The length of the string does not match the expected length
    InvalidLength,
    /// A non-base32 character was found
    InvalidChar,
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            DecodeError::InvalidLength => "invalid length",
            DecodeError::InvalidChar => "invalid character",
        };
        write!(f, "{text}")
    }
}

pub const fn decode(encoded: &str) -> Result<u128, DecodeError> {
    if encoded.len() != ULYS_LEN {
        return Err(DecodeError::InvalidLength);
    }

    let mut value: u128 = 0;

    let bytes = encoded.as_bytes();

    // Manual for loop because Range::iter() isn't const
    let mut i = 0;
    while i < ULYS_LEN {
        let val = LOOKUP[bytes[i] as usize];
        if val == NO_VALUE {
            return Err(DecodeError::InvalidChar);
        }

        value = (value << 5) | val as u128;

        i += 1;
    }

    Ok(value)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        let val = 0x4141_4141_4141_4141_4141_4141_4141_4141;
        assert_eq!(decode("21850m2ga1850m2ga1850m2ga1").unwrap(), val);
        assert_eq!(encode(val), "21850m2ga1850m2ga1850m2ga1");

        let val = 0x4d4e_3850_5144_4a59_4542_3433_5a41_3756;
        let enc = "2d9rw50ma499cmaghm6dd42dtp";
        let lower = enc.to_lowercase();
        assert_eq!(encode(val), enc);
        assert_eq!(decode(enc).unwrap(), val);
        assert_eq!(decode(&lower).unwrap(), val);
    }

    #[test]
    fn test_length() {
        assert_eq!(
            encode(0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff).len(),
            ULYS_LEN
        );
        assert_eq!(
            encode(0x0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f).len(),
            ULYS_LEN
        );
        assert_eq!(
            encode(0x0000_0000_0000_0000_0000_0000_0000_0000).len(),
            ULYS_LEN
        );

        assert_eq!(decode(""), Err(DecodeError::InvalidLength));
        assert_eq!(
            decode("2d9rw50ma499cmaghm6dd42dt"),
            Err(DecodeError::InvalidLength)
        );
        assert_eq!(
            decode("2d9rw50ma499cmaghm6dd42dtpP"),
            Err(DecodeError::InvalidLength)
        );
    }

    #[test]
    fn test_chars() {
        for ref c in encode(0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff).bytes() {
            assert!(ALPHABET.contains(c));
        }
        for ref c in encode(0x0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f).bytes() {
            assert!(ALPHABET.contains(c));
        }
        for ref c in encode(0x0000_0000_0000_0000_0000_0000_0000_0000).bytes() {
            assert!(ALPHABET.contains(c));
        }

        assert_eq!(
            decode("2d9rw50[a499cmaghm6dd42dtp"),
            Err(DecodeError::InvalidChar)
        );
        assert_eq!(
            decode("2d9rw50la499cmaghm6dd42dtp"),
            Err(DecodeError::InvalidChar)
        );
        assert_eq!(
            decode("2d9rw50ia499cmaghm6dd42dtp"),
            Err(DecodeError::InvalidChar)
        );
    }
}
