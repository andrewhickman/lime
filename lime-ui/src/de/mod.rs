mod imp;
#[cfg(test)]
mod tests;

pub use self::imp::deserialize;

use std::borrow::Cow;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de;
use specs::prelude::*;
use specs::storage::InsertResult;
use specs::world::EntitiesRes;

pub struct Seed<'de: 'a, 'a> {
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    res: &'a Resources,
    entity: Entity,
}

impl<'de, 'a> Seed<'de, 'a> {
    pub fn get_entity(&mut self, name: Cow<'de, str>) -> Entity {
        get_entity(name, &mut self.names, &*self.res.fetch())
    }
}

fn get_entity<'de, 'a>(
    name: Cow<'de, str>,
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    ents: &'a EntitiesRes,
) -> Entity {
    *names.entry(name).or_insert_with(|| ents.create())
}

#[derive(Default)]
pub struct Registry {
    map: FnvHashMap<
        &'static str,
        Box<
            for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>)
                -> Result<(), erased::Error>,
        >,
    >,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            map: FnvHashMap::default(),
        }
    }

    pub fn register<C>(&mut self, key: &'static str)
    where
        C: de::DeserializeOwned + Component,
    {
        self.register_with_deserialize_and_insert::<C, _, _>(
            key,
            default_deserialize,
            default_insert,
        )
    }

    pub fn register_with_deserialize<C, D>(&mut self, key: &'static str, deserialize: D)
    where
        C: Component,
        D: for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>)
                -> Result<C, erased::Error>
            + 'static,
    {
        self.register_with_deserialize_and_insert(key, deserialize, default_insert)
    }

    pub fn register_with_insert<C, I>(&mut self, key: &'static str, insert: I)
    where
        C: de::DeserializeOwned + 'static,
        I: Fn(&Resources, Entity, C) -> InsertResult<C> + 'static,
    {
        self.register_with_deserialize_and_insert(key, default_deserialize, insert)
    }

    pub fn register_with_deserialize_and_insert<C, D, I>(
        &mut self,
        key: &'static str,
        deserialize: D,
        insert: I,
    ) where
        D: for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>)
                -> Result<C, erased::Error>
            + 'static,
        I: Fn(&Resources, Entity, C) -> InsertResult<C> + 'static,
    {
        self.register_impl(key, move |seed, deserializer| {
            let entity = seed.entity;
            let res = seed.res;
            let comp = deserialize(seed, deserializer)?;
            if insert(res, entity, comp).unwrap().is_some() {
                Err(de::Error::custom(format!(
                    "component defined twice for entity '{:?}'",
                    entity
                )))
            } else {
                Ok(())
            }
        })
    }

    fn register_impl<F>(&mut self, key: &'static str, f: F)
    where
        F: for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>)
                -> Result<(), erased::Error>
            + 'static,
    {
        if self.map.insert(key, Box::new(f)).is_some() {
            panic!("component '{}' already added", key);
        }
    }
}

fn default_insert<C>(res: &Resources, ent: Entity, comp: C) -> InsertResult<C>
where
    C: Component,
{
    WriteStorage::<C>::fetch(res).insert(ent, comp)
}

fn default_deserialize<'de, C>(
    _: Seed<'de, '_>,
    deserializer: &mut erased::Deserializer<'de>,
) -> Result<C, erased::Error>
where
    C: de::DeserializeOwned,
{
    C::deserialize(deserializer)
}
