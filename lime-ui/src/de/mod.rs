#[cfg(test)]
pub mod tests;

mod registry;
mod seed;

pub use self::registry::{Deserialize, DeserializeAndInsert, Insert, Registry};
pub use self::seed::{ComponentSeed, Seed};

use std::borrow::Cow;
use std::error::Error;
use std::fmt;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;

use de::seed::UiSeed;
use tree::Root;

pub fn deserialize<'de, D>(
    deserializer: D,
    reg: &Registry,
    res: &mut Resources,
) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut names = FnvHashMap::default();
    let root = res.fetch::<Root>().entity();
    names.insert(Cow::Borrowed("root"), root);
    serde::DeserializeSeed::deserialize(UiSeed::new(reg, res, &mut names), deserializer)
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
