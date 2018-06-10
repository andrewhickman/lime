mod imp;
#[cfg(test)]
mod tests;

pub use self::imp::deserialize;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de;
use specs::prelude::*;
use specs::world::EntitiesRes;

#[derive(Default)]
pub struct Registry {
    map: FnvHashMap<&'static str, DeserializeComponentFn>,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            map: FnvHashMap::default(),
        }
    }

    pub fn register<C>(&mut self, key: &'static str)
    where
        C: de::DeserializeOwned + Component + Send + Sync,
    {
        self.map.insert(key, C::deserialize_component);
    }
}

pub struct Seed<'de: 'a, 'a> {
    names: &'a mut FnvHashMap<&'de str, Entity>,
    ents: &'a EntitiesRes,
    res: &'a Resources,
}

impl<'de: 'a, 'a> Seed<'de, 'a> {
    pub fn get_entity(&mut self, name: &'de str) -> Entity {
        let Seed { names, ents, .. } = self;
        *names.entry(name).or_insert_with(|| ents.create())
    }
}

pub trait DeserializeComponent {
    fn deserialize_component<'de>(
        seed: Seed<'de, '_>,
        deserializer: &mut erased::Deserializer<'de>,
        entity: Entity,
    ) -> Result<(), erased::Error>;
}

type DeserializeComponentFn =
    for<'de> fn(seed: Seed<'de, '_>, &mut erased::Deserializer<'de>, Entity)
        -> Result<(), erased::Error>;

impl<C> DeserializeComponent for C
where
    C: de::DeserializeOwned + Component + Send + Sync,
{
    fn deserialize_component<'de>(
        seed: Seed<'de, '_>,
        deserializer: &mut erased::Deserializer<'de>,
        entity: Entity,
    ) -> Result<(), erased::Error> {
        let comp = de::Deserialize::deserialize(deserializer)?;
        let mut storage = WriteStorage::<C>::fetch(&seed.res);
        storage.insert(entity, comp).unwrap(); // entity just created so this shouldn't fail.
        Ok(())
    }
}

/*
pub trait DeserializeComponentSeed: Sized {
    fn deserialize_component_seed<'de: 'a, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<Self, erased::Error>;
}

impl<C> DeserializeComponentSeed for C
where
    C: de::DeserializeOwned,
{
    fn deserialize_component_seed<'de: 'a, 'a>(
        _: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<Self, erased::Error> {
        C::deserialize(deserializer)
    }
}
*/
