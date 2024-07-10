use crate::{bitmask, Ulys};
use std::time::{Duration, SystemTime};

impl Ulys {
    /// Creates a new Ulys with the current time (UTC)
    ///
    /// Using this function to generate Ulyss will not guarantee monotonic sort order.
    /// See `ulys::Generator` for a monotonic sort order.
    /// # Example
    /// ```rust
    /// use ulys::Ulys;
    ///
    /// let my_ulys = Ulys::new();
    /// ```
    pub fn new() -> Ulys {
        Ulys::from_datetime(crate::time_utils::now())
    }

    /// Creates a new Ulys using data from the given random number generator
    ///
    /// # Example
    /// ```rust
    /// use rand::prelude::*;
    /// use ulys::Ulys;
    ///
    /// let mut rng = StdRng::from_entropy();
    /// let ulys = Ulys::with_source(&mut rng);
    /// ```
    pub fn with_source<R: rand::Rng>(source: &mut R) -> Ulys {
        Ulys::from_datetime_with_source(crate::time_utils::now(), source)
    }

    /// Creates a new Ulys with the given datetime
    ///
    /// This can be useful when migrating data to use Ulys identifiers.
    ///
    /// This will take the maximum of the `[SystemTime]` argument and `[SystemTime::UNIX_EPOCH]`
    /// as earlier times are not valid for a Ulys timestamp
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use ulys::Ulys;
    ///
    /// let ulys = Ulys::from_datetime(SystemTime::now());
    /// ```
    pub fn from_datetime(datetime: SystemTime) -> Ulys {
        Ulys::from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    /// Creates a new Ulys with the given datetime and random number generator
    ///
    /// This will take the maximum of the `[SystemTime]` argument and `[SystemTime::UNIX_EPOCH]`
    /// as earlier times are not valid for a Ulys timestamp
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use rand::prelude::*;
    /// use ulys::Ulys;
    ///
    /// let mut rng = StdRng::from_entropy();
    /// let ulys = Ulys::from_datetime_with_source(SystemTime::now(), &mut rng);
    /// ```
    pub fn from_datetime_with_source<R>(datetime: SystemTime, source: &mut R) -> Ulys
    where
        R: rand::Rng + ?Sized,
    {
        let timestamp = datetime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        let timebits = (timestamp & bitmask!(Self::TIME_BITS)) as u64;

        let msb = timebits << 16 | u64::from(source.gen::<u16>());
        let lsb = source.gen::<u64>();
        Ulys::from((msb, lsb))
    }

    /// Gets the datetime of when this Ulys was created accurate to 1ms
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use ulys::Ulys;
    ///
    /// let dt = SystemTime::now();
    /// let ulys = Ulys::from_datetime(dt);
    ///
    /// assert!(
    ///     dt + Duration::from_millis(1) >= ulys.datetime()
    ///     && dt - Duration::from_millis(1) <= ulys.datetime()
    /// );
    /// ```
    pub fn datetime(&self) -> SystemTime {
        let stamp = self.timestamp_ms();
        SystemTime::UNIX_EPOCH + Duration::from_millis(stamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic() {
        let ulys = Ulys::new();
        let encoded = ulys.to_string();
        let ulys2 = Ulys::from_string(&encoded).expect("failed to deserialize");

        println!("{encoded}");
        println!("{ulys:?}");
        println!("{ulys2:?}");
        assert_eq!(ulys, ulys2);
    }

    #[test]
    fn test_source() {
        use rand::rngs::mock::StepRng;
        let mut source = StepRng::new(123, 0);

        let u1 = Ulys::with_source(&mut source);
        let dt = SystemTime::now() + Duration::from_millis(1);
        let u2 = Ulys::from_datetime_with_source(dt, &mut source);
        let u3 = Ulys::from_datetime_with_source(dt, &mut source);

        assert!(u1 < u2);
        assert_eq!(u2, u3);
    }

    #[test]
    fn test_order() {
        let dt = SystemTime::now();
        let ulys1 = Ulys::from_datetime(dt);
        let ulys2 = Ulys::from_datetime(dt + Duration::from_millis(1));
        assert!(ulys1 < ulys2);
    }

    #[test]
    fn test_datetime() {
        let dt = SystemTime::now();
        let ulys = Ulys::from_datetime(dt);

        println!("{:?}, {:?}", dt, ulys.datetime());
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
    fn default_is_nil() {
        assert_eq!(Ulys::default(), Ulys::nil());
    }

    #[test]
    fn nil_is_at_unix_epoch() {
        assert_eq!(Ulys::nil().datetime(), SystemTime::UNIX_EPOCH);
    }

    #[test]
    fn truncates_at_unix_epoch() {
        if let Some(before_epoch) = SystemTime::UNIX_EPOCH.checked_sub(Duration::from_secs(100)) {
            assert!(before_epoch < SystemTime::UNIX_EPOCH);
            assert_eq!(
                Ulys::from_datetime(before_epoch).datetime(),
                SystemTime::UNIX_EPOCH
            );
        } else {
            // Prior dates are not representable (e.g. wasm32-wasi)
        }
    }
}
