//! Serialization and deserialization.
//!
//! By default, serialization and deserialization go through ULYSes 26-character
//! canonical string representation as set by the ULID standard.
//!
//! ULYSes can optionally be serialized as u128 integers using the `ulys_as_u128`
//! module. See the module's documentation for examples.

use crate::{Ulys, ULYS_LEN};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Ulys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buffer = [0; ULYS_LEN];
        let text = self.array_to_str(&mut buffer);
        text.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Ulys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized_str = String::deserialize(deserializer)?;
        Self::from_string(&deserialized_str).map_err(serde::de::Error::custom)
    }
}

/// Serialization and deserialization of ULYSes through their inner u128 type.
///
/// To use it, annotate a field with
/// `#[serde(with = "ulys_as_u128")]`,
/// `#[serde(serialize_with = "ulys_as_u128")]`, or
/// `#[serde(deserialize_with = "ulys_as_u128")]`.
///
/// # Examples
/// ```
/// # use ulys::Ulys;
/// # use ulys::serde::ulys_as_u128;
/// # use serde_derive::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize)]
/// struct U128Example {
///     #[serde(with = "ulys_as_u128")]
///     identifier: Ulys
/// }
/// ```
pub mod ulys_as_u128 {
    use crate::Ulys;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes a ULYS as a u128 type.
    pub fn serialize<S>(value: &Ulys, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.0.serialize(serializer)
    }

    /// Deserializes a ULYS from a u128 type.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Ulys, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized_u128 = u128::deserialize(deserializer)?;
        Ok(Ulys(deserialized_u128))
    }
}

/// Serialization and deserialization of ULYSes through UUID strings.
///
/// To use this module, annotate a field with
/// `#[serde(with = "ulys_as_uuid")]`,
/// `#[serde(serialize_with = "ulys_as_uuid")]`, or
/// `#[serde(deserialize_with = "ulys_as_uuid")]`.
///
/// # Examples
/// ```
/// # use ulys::Ulys;
/// # use ulys::serde::ulys_as_uuid;
/// # use serde_derive::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize)]
/// struct UuidExample {
///     #[serde(with = "ulys_as_uuid")]
///     identifier: Ulys
/// }
/// ```
#[cfg(all(feature = "uuid", feature = "serde"))]
pub mod ulys_as_uuid {
    use crate::Ulys;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use uuid::Uuid;

    /// Converts the ULYS to a UUID and serializes it as a string.
    pub fn serialize<S>(value: &Ulys, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uuid: Uuid = (*value).into();
        uuid.to_string().serialize(serializer)
    }

    /// Deserializes a ULYS from a string containing a UUID.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Ulys, D::Error>
    where
        D: Deserializer<'de>,
    {
        let de_string = String::deserialize(deserializer)?;
        let de_uuid = Uuid::parse_str(&de_string).map_err(serde::de::Error::custom)?;
        Ok(Ulys::from(de_uuid))
    }
}
