mod registry;
mod seed;
#[cfg(test)]
mod tests;

pub use self::registry::{Deserialize, DeserializeAndInsert, Insert, Registry};
pub use self::seed::{ComponentSeed, Seed};

use std::error::Error;
use std::fmt;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;

use de::seed::UiSeed;
use tree::Root;
use {init_dispatcher, init_world};

pub fn deserialize<'de, D>(
    deserializer: D,
    reg: &Registry,
    res: &mut Resources,
) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut names = FnvHashMap::default();
    serde::DeserializeSeed::deserialize(UiSeed::new(reg, res, &mut names), deserializer)?;
    if let Some(&entity) = names.get("root") {
        res.insert(Root::new(entity));
        Ok(())
    } else {
        Err(serde::Error::custom("no root entity defined"))
    }
}

pub fn init<'de, D>(
    world: &mut World,
    dispatcher: &mut DispatcherBuilder<'_, '_>,
    deserializer: D,
    reg: &Registry,
) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    init_world(world);
    deserialize(deserializer, reg, &mut world.res)?;
    init_dispatcher(world, dispatcher);
    Ok(())
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
