use std::any::TypeId;
use std::borrow::Cow;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;

use de::{DeserializeError, Seed};

struct Entry {
    ty: TypeId,
    de: Box<
        for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>,
    >,
}

#[derive(Default)]
pub struct Registry {
    map: FnvHashMap<&'static str, Entry>,
}

pub trait Insert: Sized + 'static {
    fn insert<'de, 'a>(self, seed: Seed<'de, 'a>) -> Result<Option<Self>, erased::Error>;
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
        use *;

        let mut reg = Registry {
            map: FnvHashMap::with_capacity_and_hasher(4, Default::default()),
        };

        reg.register_with_deserialize_and_insert::<State>("State");
        reg.register_with_deserialize_and_insert::<layout::Constraints>("Constraints");
        reg.register_with_deserialize_and_insert::<layout::Position>("Position");
        reg.register_with_deserialize::<tree::Node>("Children");
        reg.register::<draw::Brush>("Brush");
        reg.register_with_deserialize_and_insert::<draw::Style>("Style");
        reg.register::<widget::button::Button>("Button");
        reg.register::<widget::button::ButtonStyle>("ButtonStyle");
        reg.register::<widget::button::ToggleButton>("ToggleButton");
        reg.register::<widget::button::ToggleButtonStyle>("ToggleButtonStyle");
        reg.register_with_deserialize::<widget::button::RadioButton>("RadioButton");
        reg.register_with_deserialize::<widget::button::RadioButtonGroup>("RadioButtonGroup");
        reg.register::<widget::button::RadioButtonStyle>("RadioButtonStyle");
        reg.register_with_deserialize::<widget::grid::Grid>("Grid");
        reg.register_with_insert::<widget::grid::de::Row>("Row");
        reg.register_with_insert::<widget::grid::de::Col>("Col");

        reg
    }

    pub fn register<C>(&mut self, key: &'static str)
    where
        C: serde::DeserializeOwned + Component,
    {
        self.register_impl::<C, _>(
            key,
            deserialize_and_insert::<C, _, _>(default_deserialize, default_insert),
        )
    }

    pub fn register_with_deserialize<C>(&mut self, key: &'static str)
    where
        C: Deserialize + Component,
    {
        self.register_impl::<C, _>(key, deserialize_and_insert(C::deserialize, default_insert))
    }

    pub fn register_with_insert<C>(&mut self, key: &'static str)
    where
        C: serde::DeserializeOwned + Insert,
    {
        self.register_impl::<C, _>(key, deserialize_and_insert(default_deserialize, C::insert))
    }

    pub fn register_with_deserialize_and_insert<C>(&mut self, key: &'static str)
    where
        C: DeserializeAndInsert,
    {
        self.register_impl::<C, _>(key, C::deserialize_and_insert)
    }

    fn register_impl<C, F>(&mut self, key: &'static str, f: F)
    where
        C: 'static,
        F: for<'de, 'a> Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>)
                -> Result<(), erased::Error>
            + 'static,
    {
        let entry = Entry {
            de: Box::new(f),
            ty: TypeId::of::<C>(),
        };
        if self.map.insert(key, entry).is_some() {
            panic!("component '{}' already added", key);
        }
    }

    pub fn get_de<'de, 'a>(
        &'a self,
        key: Cow<'de, str>,
    ) -> Result<
        &'a Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>,
        DeserializeError,
    > {
        if let Some(entry) = self.map.get(key.as_ref()) {
            Ok(&*entry.de)
        } else {
            Err(DeserializeError(format!("key '{}' not in registry", key)))
        }
    }

    pub fn get_ty(&self, key: Cow<str>) -> Result<TypeId, DeserializeError> {
        if let Some(entry) = self.map.get(key.as_ref()) {
            Ok(entry.ty)
        } else {
            Err(DeserializeError(format!("key '{}' not in registry", key)))
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
    I: for<'de, 'a> Fn(C, Seed<'de, 'a>) -> Result<Option<C>, erased::Error> + 'static,
{
    move |mut seed, deserializer| {
        let comp = deserialize(seed.borrow(), deserializer)?;
        seed.insert_with(comp, &insert)?;
        Ok(())
    }
}

pub(in de) fn default_insert<C>(comp: C, seed: Seed<'_, '_>) -> Result<Option<C>, erased::Error>
where
    C: Component,
{
    WriteStorage::<C>::fetch(seed.res)
        .insert(seed.entity, comp)
        .map_err(serde::Error::custom)
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
