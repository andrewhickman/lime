mod imp;
#[cfg(test)]
mod tests;

pub use self::imp::deserialize;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de;
use specs::prelude::*;
use specs::world::EntitiesRes;

pub struct Seed<'de: 'a, 'a> {
    names: &'a mut FnvHashMap<&'de str, Entity>,
    ents: &'a EntitiesRes,
    res: &'a Resources,
}

impl<'de, 'a> Seed<'de, 'a> {
    pub fn get_entity(&mut self, name: &'de str) -> Entity {
        get_entity(name, &mut self.names, &self.ents)
    }
}

fn get_entity<'de, 'a>(
    name: &'de str,
    names: &'a mut FnvHashMap<&'de str, Entity>,
    ents: &'a EntitiesRes,
) -> Entity {
    *names.entry(name).or_insert_with(|| ents.create())
}

pub trait DeserializeComponent: Sized {
    fn deserialize<'de, 'a, D>(seed: Seed<'de, 'a>, deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>;
}

impl<C> DeserializeComponent for C
where
    C: de::DeserializeOwned,
{
    fn deserialize<'de, 'a, D>(_: Seed<'de, 'a>, deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        C::deserialize(deserializer)
    }
}

type DeserializeComponentFn =
    for<'de, 'a> fn(seed: Seed<'de, 'a>, &mut erased::Deserializer<'de>, Entity)
        -> Result<(), erased::Error>;

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
        C: DeserializeComponent + Component + Send + Sync,
    {
        fn deserialize_component<'de, 'a, C>(
            seed: Seed<'de, 'a>,
            deserializer: &mut erased::Deserializer<'de>,
            entity: Entity,
        ) -> Result<(), erased::Error>
        where
            C: DeserializeComponent + Component + Send + Sync,
        {
            let res = seed.res;
            let comp = DeserializeComponent::deserialize(seed, deserializer)?;
            let mut storage = WriteStorage::<C>::fetch(res);
            if storage.insert(entity, comp).unwrap().is_some() {
                return Err(de::Error::custom(format!(
                    "component defined twice for entity '{:?}'",
                    entity
                )));
            }
            Ok(())
        }

        if self.map.insert(key, deserialize_component::<C>).is_some() {
            panic!("component '{}' already added", key);
        }
    }
}
