use std::borrow::Cow;
use std::fmt;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;
use specs::world::EntitiesRes;

use de::{registry, DeserializeError, Registry};

pub struct Seed<'de: 'a, 'a> {
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    pub res: &'a Resources,
    pub reg: &'a Registry,
    pub entity: Entity,
    pub parent: Option<Entity>,
}

impl<'de, 'a> Seed<'de, 'a> {
    pub fn get_entity(&mut self, name: Cow<'de, str>) -> Result<Entity, DeserializeError> {
        if is_valid_name(name.as_ref()) {
            Ok(get_entity(name, &mut self.names, &*self.res.fetch()))
        } else {
            Err(DeserializeError(format!("invalid name '{}'", name)))
        }
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
            parent: self.parent,
        }
    }

    pub fn component_seed(
        self,
        key: Cow<'de, str>,
    ) -> Result<ComponentSeed<'de, 'a>, DeserializeError> {
        let de = self.reg.get_de(key)?;
        Ok(ComponentSeed { seed: self, de })
    }

    pub(crate) fn entity_seed(self, entity: Entity, parent: Entity) -> EntitySeed<'de, 'a> {
        EntitySeed(Seed {
            names: self.names,
            res: self.res,
            reg: self.reg,
            entity,
            parent: Some(parent),
        })
    }

    pub fn insert<C>(&mut self, comp: C) -> Result<(), DeserializeError>
    where
        C: Component,
    {
        self.insert_with(comp, registry::default_insert)
    }

    pub fn insert_with<C, I>(&mut self, comp: C, insert: I) -> Result<(), DeserializeError>
    where
        I: for<'de2, 'a2> Fn(C, Seed<'de2, 'a2>) -> Result<Option<C>, erased::Error>,
    {
        if insert(comp, self.borrow()).unwrap().is_some() {
            Err(DeserializeError(format!(
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
                    map.next_value_seed(EntitySeed(Seed {
                        res: self.0.res,
                        reg: self.0.reg,
                        names: self.0.names,
                        entity,
                        parent: None,
                    }))?;
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}

pub(crate) struct EntitySeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl<'de: 'a, 'a> serde::DeserializeSeed<'de> for EntitySeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(Seed<'de, 'a>);

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
                    map.next_value_seed(self.0
                        .borrow()
                        .component_seed(key)
                        .map_err(serde::Error::custom)?)?;
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self.0))
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

fn is_valid_name(s: &str) -> bool {
    s.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
}
