// src/postgres.rs

//! Conversion to and from the `PostgreSQL` `UUID` type.

use crate::Ulys;
use bytes::BufMut as _;
use bytes::BytesMut;
use postgres_types::accepts;
use postgres_types::to_sql_checked;
use postgres_types::{FromSql, IsNull, ToSql, Type};
use std::error::Error;

impl FromSql<'_> for Ulys {
	fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
		if raw.len() != 16 {
			return Err("invalid message length: uuid size mismatch".into());
		}
		let mut bytes = [0; 16];
		bytes.copy_from_slice(raw);
		Ok(Ulys(u128::from_be_bytes(bytes)))
	}
	accepts!(UUID);
}

impl ToSql for Ulys {
	fn to_sql(&self, _: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
		let bytes: u128 = self.0;
		w.put_slice(&bytes.to_be_bytes());
		Ok(IsNull::No)
	}

	accepts!(UUID);
	to_sql_checked!();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Ulys;
	use postgres_types::Type;
	use std::io::Read as _;

	#[test]
	fn postgres_cycle() {
		let ulys = Ulys::from_string("068dkwmn3a441g20mzbsmyk5b8").unwrap();

		let mut buf = bytes::BytesMut::new();
		let ty = &Type::UUID;
		let _ = ulys.to_sql(ty, &mut buf);

		assert_eq!(16, buf.len());

		let bs = buf.bytes().map(|byte| byte.unwrap()).collect::<Vec<u8>>();

		assert_eq!(ulys, Ulys::from_sql(ty, &bs).unwrap());
	}
}
