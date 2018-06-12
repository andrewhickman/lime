use std::borrow::Cow;
use std::fmt;

use fnv::FnvHashMap;
use serde::de as serde;
use specs::prelude::*;

use de::{get_entity, Registry, Seed};

pub fn deserialize<'de, D>(deserializer: D, reg: &Registry, res: &Resources) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde::DeserializeSeed::deserialize(
        UiSeed {
            reg,
            res,
            names: &mut FnvHashMap::default(),
        },
        deserializer,
    )
}

struct UiSeed<'de: 'a, 'a> {
    res: &'a Resources,
    reg: &'a Registry,
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
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
                        res: self.0.res,
                        reg: self.0.reg,
                        names: self.0.names,
                        entity,
                    })?;
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}

struct EntitySeed<'de: 'a, 'a> {
    res: &'a Resources,
    reg: &'a Registry,
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    entity: Entity,
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

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<Cow<str>>()? {
                    map.next_value_seed(Seed {
                        entity: self.0.entity,
                        res: self.0.res,
                        reg: self.0.reg,
                        names: self.0.names,
                    }.get_deserialize_fn(key)?)?;
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}
