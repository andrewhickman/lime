mod registry;
mod seed;
#[cfg(test)]
mod tests;

pub use self::registry::{Deserialize, DeserializeAndInsert, Insert, Registry};
pub use self::seed::{ComponentSeed, EntitySeed, Seed};

use de::seed::UiSeed;

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
