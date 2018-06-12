mod imp;
mod registry;
#[cfg(test)]
mod tests;

pub use self::imp::deserialize;
pub use self::registry::{Deserialize, DeserializeAndInsert, Insert, Registry};

use std::borrow::Cow;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;
use specs::storage::InsertResult;
use specs::world::EntitiesRes;

pub struct Seed<'de: 'a, 'a> {
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    pub res: &'a Resources,
    pub reg: &'a Registry,
    pub entity: Entity,
}

impl<'de, 'a> Seed<'de, 'a> {
    pub fn get_entity(&mut self, name: Cow<'de, str>) -> Entity {
        get_entity(name, &mut self.names, &*self.res.fetch())
    }

    pub fn get_deserialize_fn<E>(self, key: Cow<'de, str>) -> Result<DeserializeFn<'de, 'a>, E>
    where
        E: serde::Error,
    {
        let de = self.reg.get(key)?;
        Ok(DeserializeFn { seed: self, de })
    }

    pub fn insert<C, E>(&mut self, comp: C) -> Result<(), E>
    where
        C: Component,
        E: serde::Error,
    {
        self.insert_with(comp, registry::default_insert)
    }

    pub fn insert_with<C, I, E>(&mut self, comp: C, insert: I) -> Result<(), E>
    where
        I: Fn(C, Entity, &Resources) -> InsertResult<C>,
        E: serde::Error,
    {
        if insert(comp, self.entity, self.res).unwrap().is_some() {
            Err(serde::Error::custom(format!(
                "component defined twice for entity '{:?}'",
                self.entity
            )))
        } else {
            Ok(())
        }
    }
}

pub struct DeserializeFn<'de: 'a, 'a> {
    seed: Seed<'de, 'a>,
    de: &'a Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>,
}

impl<'de, 'a> serde::DeserializeSeed<'de> for DeserializeFn<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        (self.de)(self.seed, &mut erased::Deserializer::erase(deserializer))
            .map_err(serde::Error::custom)
    }
}

fn get_entity<'de, 'a>(
    name: Cow<'de, str>,
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    ents: &'a EntitiesRes,
) -> Entity {
    *names.entry(name).or_insert_with(|| ents.create())
}
