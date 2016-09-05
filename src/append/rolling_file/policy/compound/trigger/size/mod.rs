//! The size trigger.

use serde::de;
use std::ascii::AsciiExt;
use std::error::Error;

use append::rolling_file::LogFile;
use append::rolling_file::policy::compound::trigger::Trigger;
use file::{Deserialize, Deserializers};

include!("config.rs");

fn deserialize_limit<D>(d: &mut D) -> Result<u64, D::Error>
    where D: de::Deserializer
{
    struct V;

    impl de::Visitor for V {
        type Value = u64;

        fn visit_u64<E>(&mut self, v: u64) -> Result<u64, E>
            where E: de::Error
        {
            Ok(v)
        }

        fn visit_i64<E>(&mut self, v: i64) -> Result<u64, E>
            where E: de::Error
        {
            if v < 0 {
                return Err(E::invalid_value("must be non-negative"));
            }

            Ok(v as u64)
        }

        fn visit_str<E>(&mut self, v: &str) -> Result<u64, E>
            where E: de::Error
        {
            let (number, unit) = match v.find(|c: char| !c.is_digit(10)) {
                Some(n) => (v[..n].trim(), Some(v[n..].trim())),
                None => (v.trim(), None),
            };

            let number = match number.parse::<u64>() {
                Ok(n) => n,
                Err(e) => return Err(E::invalid_value(&e.to_string())),
            };

            let unit = match unit {
                Some(u) => u,
                None => return Ok(number),
            };

            let number =
                if unit.eq_ignore_ascii_case("b") {
                    Some(number)
                } else if unit.eq_ignore_ascii_case("kb") || unit.eq_ignore_ascii_case("kib") {
                    number.checked_mul(1024)
                } else if unit.eq_ignore_ascii_case("mb") || unit.eq_ignore_ascii_case("mib") {
                    number.checked_mul(1024 * 1024)
                } else if unit.eq_ignore_ascii_case("gb") || unit.eq_ignore_ascii_case("gib") {
                    number.checked_mul(1024 * 1024 * 1024)
                } else if unit.eq_ignore_ascii_case("tb") || unit.eq_ignore_ascii_case("tib") {
                    number.checked_mul(1024 * 1024 * 1024 * 1024)
                } else {
                    return Err(E::invalid_value(&format!("invalid unit `{}`", unit)));
                };

            match number {
                Some(n) => Ok(n),
                None => Err(E::invalid_value("value overflowed")),
            }
        }
    }

    d.deserialize(V)
}

/// A trigger which rolls the log once it has passed a certain size.
#[derive(Debug)]
pub struct SizeTrigger {
    limit: u64,
}

impl SizeTrigger {
    /// Returns a new trigger which rolls the log once it has passed the
    /// specified size in bytes.
    pub fn new(limit: u64) -> SizeTrigger {
        SizeTrigger { limit: limit }
    }
}

impl Trigger for SizeTrigger {
    fn trigger(&self, file: &LogFile) -> Result<bool, Box<Error>> {
        Ok(file.len() > self.limit)
    }
}

/// A deserializer for the `SizeTrigger`.
///
/// # Configuration
///
/// ```yaml
/// kind: size
///
/// # The size limit in bytes. The following units are supported (case insensitive):
/// # "b", "kb", "kib", "mb", "mib", "gb", "gib", "tb", "tib". The unit defaults to
/// # bytes if not specified. Required.
/// limit: 10 mb
/// ```
pub struct SizeTriggerDeserializer;

impl Deserialize for SizeTriggerDeserializer {
    type Trait = Trigger;

    type Config = SizeTriggerConfig;

    fn deserialize(&self,
                   config: SizeTriggerConfig,
                   _: &Deserializers)
                   -> Result<Box<Trigger>, Box<Error>> {
        Ok(Box::new(SizeTrigger::new(config.limit)))
    }
}
