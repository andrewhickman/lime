mod registry;
mod seed;
#[cfg(test)]
mod tests;

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
    deserialize_with_names(deserializer, reg, res, &mut FnvHashMap::default())
}

pub fn deserialize_with_names<'de, D>(
    deserializer: D,
    reg: &Registry,
    res: &mut Resources,
    names: &mut FnvHashMap<Cow<'de, str>, Entity>,
) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let root = res.fetch::<Root>().entity();
    names.insert(Cow::Borrowed("root"), root);
    serde::DeserializeSeed::deserialize(UiSeed::new(reg, res, names), deserializer)
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
