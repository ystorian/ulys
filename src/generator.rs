use std::time::{Duration, SystemTime};

use std::fmt;

use crate::Ulys;

/// A Ulys generator that provides monotonically increasing Ulyses
pub struct Generator {
    previous: Ulys,
}

impl Generator {
    /// Create a new ulys generator for monotonically ordered Ulyses
    ///
    /// # Example
    /// ```rust
    /// use ulys::Generator;
    ///
    /// let mut gen = Generator::new();
    ///
    /// let ulys1 = gen.generate().unwrap();
    /// let ulys2 = gen.generate().unwrap();
    ///
    /// assert!(ulys1 < ulys2);
    /// ```
    pub fn new() -> Generator {
        Generator {
            previous: Ulys::nil(),
        }
    }

    /// Generate a new Ulys. Each call is guaranteed to provide a Ulys with a larger value than the
    /// last call. If the random bits would overflow, this method will return an error.
    ///
    /// ```rust
    /// use ulys::Generator;
    /// let mut gen = Generator::new();
    ///
    /// let ulys1 = gen.generate().unwrap();
    /// let ulys2 = gen.generate().unwrap();
    ///
    /// assert!(ulys1 < ulys2);
    /// ```
    pub fn generate(&mut self) -> Result<Ulys, MonotonicError> {
        self.generate_from_datetime(crate::time_utils::now())
    }

    /// Generate a new Ulys matching the given DateTime.
    /// Each call is guaranteed to provide a Ulys with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulys::Generator;
    /// use std::time::SystemTime;
    ///
    /// let dt = SystemTime::now();
    /// let mut gen = Generator::new();
    ///
    /// let ulys1 = gen.generate_from_datetime(dt).unwrap();
    /// let ulys2 = gen.generate_from_datetime(dt).unwrap();
    ///
    /// assert_eq!(ulys1.datetime(), ulys2.datetime());
    /// assert!(ulys1 < ulys2);
    /// ```
    pub fn generate_from_datetime(&mut self, datetime: SystemTime) -> Result<Ulys, MonotonicError> {
        self.generate_from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    /// Generate a new monotonic increasing Ulys with the given source
    /// Each call is guaranteed to provide a Ulys with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulys::Generator;
    /// use ulys::Ulys;
    /// use std::time::SystemTime;
    /// use rand::prelude::*;
    ///
    /// let mut rng = StdRng::from_entropy();
    /// let mut gen = Generator::new();
    ///
    /// let ulys1 = gen.generate_with_source(&mut rng).unwrap();
    /// let ulys2 = gen.generate_with_source(&mut rng).unwrap();
    ///
    /// assert!(ulys1 < ulys2);
    /// ```
    pub fn generate_with_source<R>(&mut self, source: &mut R) -> Result<Ulys, MonotonicError>
    where
        R: rand::Rng + ?Sized,
    {
        self.generate_from_datetime_with_source(crate::time_utils::now(), source)
    }

    /// Generate a new monotonic increasing Ulys with the given source matching the given DateTime
    /// Each call is guaranteed to provide a Ulys with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulys::Generator;
    /// use std::time::SystemTime;
    /// use rand::prelude::*;
    ///
    /// let dt = SystemTime::now();
    /// let mut rng = StdRng::from_entropy();
    /// let mut gen = Generator::new();
    ///
    /// let ulys1 = gen.generate_from_datetime_with_source(dt, &mut rng).unwrap();
    /// let ulys2 = gen.generate_from_datetime_with_source(dt, &mut rng).unwrap();
    ///
    /// assert_eq!(ulys1.datetime(), ulys2.datetime());
    /// assert!(ulys1 < ulys2);
    /// ```
    pub fn generate_from_datetime_with_source<R>(
        &mut self,
        datetime: SystemTime,
        source: &mut R,
    ) -> Result<Ulys, MonotonicError>
    where
        R: rand::Rng + ?Sized,
    {
        let last_ms = self.previous.timestamp_ms();
        // maybe time went backward, or it is the same ms.
        // increment instead of generating a new random so that it is monotonic
        if datetime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis()
            <= u128::from(last_ms)
        {
            if let Some(next) = self.previous.increment() {
                self.previous = next;
                return Ok(next);
            } else {
                return Err(MonotonicError::Overflow);
            }
        }
        let next = Ulys::from_datetime_with_source(datetime, source);
        self.previous = next;
        Ok(next)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

/// Error while trying to generate a monotonic increment in the same millisecond
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum MonotonicError {
    /// Would overflow into the next millisecond
    Overflow,
}

impl std::error::Error for MonotonicError {}

impl fmt::Display for MonotonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            MonotonicError::Overflow => "Ulys random bits would overflow",
        };
        write!(f, "{}", text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_order_monotonic() {
        let dt = SystemTime::now();
        let mut gen = Generator::new();
        let ulys1 = gen.generate_from_datetime(dt).unwrap();
        let ulys2 = gen.generate_from_datetime(dt).unwrap();
        let ulys3 = Ulys::from_datetime(dt + Duration::from_millis(1));
        assert_eq!(ulys1.0 + 1, ulys2.0);
        assert!(ulys2 < ulys3);
        assert!(ulys2.timestamp_ms() < ulys3.timestamp_ms())
    }

    #[test]
    fn test_order_monotonic_with_source() {
        use rand::rngs::mock::StepRng;
        let mut source = StepRng::new(123, 0);
        let mut gen = Generator::new();

        let _has_default = Generator::default();

        let ulys1 = gen.generate_with_source(&mut source).unwrap();
        let ulys2 = gen.generate_with_source(&mut source).unwrap();
        assert!(ulys1 < ulys2);
    }

    #[test]
    fn can_display_things() {
        println!("{}", MonotonicError::Overflow);
    }
}
