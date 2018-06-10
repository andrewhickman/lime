use std::fmt;

use erased_serde as erased;
use fnv::FnvHashMap;
use serde::de;
use specs::prelude::*;
use specs::world::EntitiesRes;

use de::{DeserializeComponentFn, Registry, Seed};

pub fn deserialize<'de, D>(
    deserializer: D,
    registry: &Registry,
    res: &Resources,
) -> Result<(), D::Error>
where
    D: de::Deserializer<'de>,
{
    de::DeserializeSeed::deserialize(
        UiSeed {
            reg: &registry.map,
            res,
            names: &mut FnvHashMap::default(),
        },
        deserializer,
    )
}

struct UiSeed<'de: 'a, 'a> {
    res: &'a Resources,
    reg: &'a FnvHashMap<&'static str, DeserializeComponentFn>,
    names: &'a mut FnvHashMap<&'de str, Entity>,
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

            fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                while seq.next_element_seed(EntitySeed {
                    res: &self.0.res,
                    reg: &self.0.reg,
                    names: &mut self.0.names,
                })?
                    .is_some()
                {}
                Ok(())
            }
        }

        deserializer.deserialize_seq(Visitor(self))
    }
}

struct EntitySeed<'de: 'a, 'a> {
    res: &'a Resources,
    reg: &'a FnvHashMap<&'static str, DeserializeComponentFn>,
    names: &'a mut FnvHashMap<&'de str, Entity>,
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

            fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let entity = self.0.res.fetch::<EntitiesRes>().create();
                while let Some(key) = map.next_key::<&str>()? {
                    if let Some(&de) = self.0.reg.get(key) {
                        map.next_value_seed(ComponentSeed {
                            entity,
                            res: self.0.res,
                            de,
                            names: &mut self.0.names,
                        })?;
                    } else {
                        return Err(<A::Error as de::Error>::custom("key not in registry"));
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
    names: &'a mut FnvHashMap<&'de str, Entity>,
    de: DeserializeComponentFn,
}

impl<'de: 'a, 'a> de::DeserializeSeed<'de> for ComponentSeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut deserializer = erased::Deserializer::erase(deserializer);
        (self.de)(
            Seed {
                names: &mut self.names,
                ents: &*self.res.fetch(),
                res: &self.res,
            },
            &mut deserializer,
            self.entity,
        ).map_err(<D::Error as de::Error>::custom)
    }
}
