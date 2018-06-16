mod registry;
mod seed;
#[cfg(test)]
mod tests;

pub use self::registry::{Deserialize, DeserializeAndInsert, Insert, Registry};
pub use self::seed::{ComponentSeed, EntitySeed, Seed};

use std::error::Error;
use std::fmt;

use de::seed::UiSeed;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;

pub fn deserialize<'de, D>(deserializer: D, reg: &Registry, res: &Resources) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde::DeserializeSeed::deserialize(
        UiSeed::new(reg, res, &mut FnvHashMap::default()),
        deserializer,
    )
}

pub fn is_valid_name(s: &str) -> bool {
    s.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[derive(Debug)]
pub struct DeserializeError(String);

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for DeserializeError {}

impl From<DeserializeError> for erased::Error {
    fn from(err: DeserializeError) -> Self {
        serde::Error::custom(err)
    }
}
