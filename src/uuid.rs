//! Conversions between ULYS and UUID.

use crate::Ulys;
use uuid::Uuid;

impl From<Uuid> for Ulys {
    fn from(uuid: Uuid) -> Self {
        Ulys(uuid.as_u128())
    }
}

impl From<Ulys> for Uuid {
    fn from(ulys: Ulys) -> Self {
        Uuid::from_u128(ulys.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn uuid_cycle() {
        let ulys = Ulys::new();
        let uuid: Uuid = ulys.into();
        let ulys2: Ulys = uuid.into();

        assert_eq!(ulys, ulys2);
    }

    #[test]
    fn uuid_str_cycle() {
        let uuid_txt = "881a3bfe-01e9-4438-a68e-b1e7e82b7f9c";
        let ulys_txt = "h0d3qzg1x523h9mep7kygavzkg";

        let ulys: Ulys = Uuid::parse_str(uuid_txt).unwrap().into();
        assert_eq!(ulys.to_string(), ulys_txt);

        let uuid: Uuid = ulys.into();
        assert_eq!(uuid.to_string(), uuid_txt);
    }
}
