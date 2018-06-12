use std::borrow::Cow;
use std::fmt;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de;
use specs::prelude::*;

use de::{get_entity, Registry, Seed};

pub fn deserialize<'de, D>(deserializer: D, reg: &Registry, res: &Resources) -> Result<(), D::Error>
where
    D: de::Deserializer<'de>,
{
    de::DeserializeSeed::deserialize(
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

impl<'de: 'a, 'a> de::DeserializeSeed<'de> for UiSeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(UiSeed<'de, 'a>);

        impl<'de, 'a> de::Visitor<'de> for Visitor<'de, 'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of entities")
            }

            fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
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

impl<'de: 'a, 'a> de::DeserializeSeed<'de> for EntitySeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(EntitySeed<'de, 'a>);

        impl<'de, 'a> de::Visitor<'de> for Visitor<'de, 'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "map of strings to components")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<Cow<str>>()? {
                    if let Some(de) = self.0.reg.map.get(key.as_ref()) {
                        map.next_value_seed(ComponentSeed {
                            entity: self.0.entity,
                            res: self.0.res,
                            de: &**de,
                            names: self.0.names,
                        })?;
                    } else {
                        return Err(de::Error::custom(format!("key '{}' not in registry", key)));
                    }
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}

pub struct ComponentSeed<'de: 'a, 'a> {
    entity: Entity,
    res: &'a Resources,
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    de: &'a Fn(Seed<'de, 'a>, &mut erased::Deserializer<'de>) -> Result<(), erased::Error>,
}

impl<'de: 'a, 'a> de::DeserializeSeed<'de> for ComponentSeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut deserializer = erased::Deserializer::erase(deserializer);
        (self.de)(
            Seed {
                entity: self.entity,
                names: self.names,
                res: self.res,
            },
            &mut deserializer,
        ).map_err(de::Error::custom)
    }
}
