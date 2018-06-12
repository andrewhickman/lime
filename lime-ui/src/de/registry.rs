use std::borrow::Cow;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
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

pub trait DeserializeAndInsert: 'static {
    fn deserialize_and_insert<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error>;
}

impl<C> DeserializeAndInsert for C
where
    C: Deserialize + Insert,
{
    fn deserialize_and_insert<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        deserialize_and_insert(C::deserialize, C::insert)(seed, deserializer)
    }
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            map: FnvHashMap::default(),
        }
    }

    pub fn register<C>(&mut self, key: &'static str)
    where
        C: serde::DeserializeOwned + Component,
    {
        self.register_impl(
            key,
            deserialize_and_insert::<C, _, _>(default_deserialize, default_insert),
        )
    }

    pub fn register_with_deserialize<C>(&mut self, key: &'static str)
    where
        C: Deserialize + Component,
    {
        self.register_impl(key, deserialize_and_insert(C::deserialize, default_insert))
    }

    pub fn register_with_insert<C>(&mut self, key: &'static str)
    where
        C: serde::DeserializeOwned + Insert,
    {
        self.register_impl(key, deserialize_and_insert(default_deserialize, C::insert))
    }

    pub fn register_with_deserialize_and_insert<C>(&mut self, key: &'static str)
    where
        C: DeserializeAndInsert,
    {
        self.register_impl(key, C::deserialize_and_insert)
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

    pub(in de) fn get<'de, 'a, E>(
        &'a self,
        key: Cow<'de, str>,
    ) -> Result<&'a Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>, E>
    where
        E: serde::Error,
    {
        if let Some(de) = self.map.get(key.as_ref()) {
            Ok(&**de)
        } else {
            Err(serde::Error::custom(format!(
                "key '{}' not in registry",
                key
            )))
        }
    }
}

pub(in de) fn deserialize_and_insert<C, D, I>(
    deserialize: D,
    insert: I,
) -> impl for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>
         + 'static
where
    D: for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<C, erased::Error>
        + 'static,
    I: Fn(C, Entity, &Resources) -> InsertResult<C> + 'static,
{
    move |mut seed, deserializer| {
        let comp = deserialize(
            Seed {
                names: seed.names,
                res: seed.res,
                reg: seed.reg,
                entity: seed.entity,
            },
            deserializer,
        )?;
        seed.insert_with(comp, &insert)
    }
}

pub(in de) fn default_insert<C>(comp: C, ent: Entity, res: &Resources) -> InsertResult<C>
where
    C: Component,
{
    WriteStorage::<C>::fetch(res).insert(ent, comp)
}

pub(in de) fn default_deserialize<'de, C>(
    _: Seed<'de, '_>,
    deserializer: &mut erased::Deserializer<'de>,
) -> Result<C, erased::Error>
where
    C: serde::DeserializeOwned,
{
    C::deserialize(deserializer)
}
