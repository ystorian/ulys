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
        let uuid_txt = "771a3bce-02e9-4428-a68e-b1e7e82b7f9f";
        let ulys_txt = "3q38xww0q98gmad3nhwzm2pzwz";

        let ulys: Ulys = Uuid::parse_str(uuid_txt).unwrap().into();
        assert_eq!(ulys.to_string(), ulys_txt);

        let uuid: Uuid = ulys.into();
        assert_eq!(uuid.to_string(), uuid_txt);
    }
}
