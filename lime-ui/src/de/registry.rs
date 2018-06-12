use std::borrow::Cow;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de;
use specs::prelude::*;
use specs::storage::InsertResult;

use de::Seed;

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

pub trait Insert: Sized + 'static {
    fn insert(self, entity: Entity, res: &Resources) -> InsertResult<Self>;
}

pub trait Deserialize: Sized + 'static {
    fn deserialize<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<Self, erased::Error>;
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
        self.register_impl::<C, _, _>(key, default_deserialize, default_insert)
    }

    pub fn register_with_deserialize<C>(&mut self, key: &'static str)
    where
        C: Deserialize + Component,
    {
        self.register_impl(key, C::deserialize, default_insert)
    }

    pub fn register_with_insert<C>(&mut self, key: &'static str)
    where
        C: de::DeserializeOwned + Insert,
    {
        self.register_impl(key, default_deserialize, C::insert)
    }

    pub fn register_with_deserialize_and_insert<C>(&mut self, key: &'static str)
    where
        C: Deserialize + Insert,
    {
        self.register_impl(key, C::deserialize, C::insert)
    }

    fn register_impl<C, D, I>(&mut self, key: &'static str, deserialize: D, insert: I)
    where
        D: for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>)
                -> Result<C, erased::Error>
            + 'static,
        I: Fn(C, Entity, &Resources) -> InsertResult<C> + 'static,
    {
        self._register_impl(key, move |seed, deserializer| {
            let entity = seed.entity;
            let res = seed.res;
            let comp = deserialize(seed, deserializer)?;
            if insert(comp, entity, res).unwrap().is_some() {
                Err(de::Error::custom(format!(
                    "component defined twice for entity '{:?}'",
                    entity
                )))
            } else {
                Ok(())
            }
        })
    }

    fn _register_impl<F>(&mut self, key: &'static str, f: F)
    where
        F: for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>)
                -> Result<(), erased::Error>
            + 'static,
    {
        if self.map.insert(key, Box::new(f)).is_some() {
            panic!("component '{}' already added", key);
        }
    }

    pub(in de) fn get<'de, 'a, E>(
        &'a self,
        key: Cow<'de, str>,
    ) -> Result<&'a Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>, E>
    where
        E: de::Error,
    {
        if let Some(de) = self.map.get(key.as_ref()) {
            Ok(&**de)
        } else {
            Err(de::Error::custom(format!("key '{}' not in registry", key)))
        }
    }
}

fn default_insert<C>(comp: C, ent: Entity, res: &Resources) -> InsertResult<C>
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
