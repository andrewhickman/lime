use std::borrow::Cow;
use std::fmt;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;
use specs::storage::InsertResult;
use specs::world::EntitiesRes;

use de::{registry, Registry};

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

    /// Reverse lookup of name given entity for error messages. Panics if entity was not created
    /// by `get_entity`.
    pub fn get_name(&self, ent: Entity) -> &str {
        self.names
            .iter()
            .find(|kv| kv.1 == &ent)
            .expect("entity not in name map")
            .0
    }

    pub fn borrow<'b>(&'b mut self) -> Seed<'de, 'b> {
        Seed {
            names: self.names,
            res: self.res,
            reg: self.reg,
            entity: self.entity,
        }
    }

    pub fn with_entity<'b>(&'b mut self, entity: Entity) -> Seed<'de, 'b> {
        Seed {
            names: self.names,
            res: self.res,
            reg: self.reg,
            entity,
        }
    }

    pub fn component_seed<E>(self, key: Cow<'de, str>) -> Result<ComponentSeed<'de, 'a>, E>
    where
        E: serde::Error,
    {
        let de = self.reg.get_de(key)?;
        Ok(ComponentSeed { seed: self, de })
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
                "component defined twice for entity '{}'",
                self.get_name(self.entity)
            )))
        } else {
            Ok(())
        }
    }
}

pub(in de) struct UiSeed<'de: 'a, 'a> {
    res: &'a Resources,
    reg: &'a Registry,
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
}

impl<'de, 'a> UiSeed<'de, 'a> {
    pub(in de) fn new(
        reg: &'a Registry,
        res: &'a Resources,
        names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    ) -> Self {
        UiSeed { reg, res, names }
    }
}

impl<'de: 'a, 'a> serde::DeserializeSeed<'de> for UiSeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(UiSeed<'de, 'a>);

        impl<'de, 'a> serde::Visitor<'de> for Visitor<'de, 'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of entities")
            }

            fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::MapAccess<'de>,
            {
                while let Some(name) = map.next_key::<Cow<str>>()? {
                    let entity = get_entity(name, &mut self.0.names, &*self.0.res.fetch());
                    map.next_value_seed(EntitySeed {
                        seed: Seed {
                            res: self.0.res,
                            reg: self.0.reg,
                            names: self.0.names,
                            entity,
                        },
                    })?;
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}

pub struct EntitySeed<'de: 'a, 'a> {
    seed: Seed<'de, 'a>,
}

impl<'de: 'a, 'a> serde::DeserializeSeed<'de> for EntitySeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(EntitySeed<'de, 'a>);

        impl<'de, 'a> serde::Visitor<'de> for Visitor<'de, 'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "map of strings to components")
            }

            fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<Cow<str>>()? {
                    map.next_value_seed(self.0.seed.borrow().component_seed(key)?)?;
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}

pub struct ComponentSeed<'de: 'a, 'a> {
    seed: Seed<'de, 'a>,
    de: &'a Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>,
}

impl<'de, 'a> serde::DeserializeSeed<'de> for ComponentSeed<'de, 'a> {
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
