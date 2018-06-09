use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

use erased_serde as erased;
use failure;
use fnv::FnvHashMap;
use serde::de;
use specs::prelude::*;
use specs::world::EntitiesRes;

pub fn deserialize<'de, D>(
    deserializer: D,
    registry: &Registry,
    res: &Resources,
) -> Result<(), D::Error>
where
    D: de::Deserializer<'de>,
{
    de::DeserializeSeed::deserialize(
        UiSeed(Inner {
            map: &registry.map,
            res,
        }),
        deserializer,
    )
}

#[derive(Default)]
pub struct Registry {
    map: FnvHashMap<Cow<'static, str>, Box<DeserializeComponent>>,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            map: FnvHashMap::default(),
        }
    }

    pub fn register<S, C>(&mut self, key: S)
    where
        S: Into<Cow<'static, str>>,
        C: de::DeserializeOwned + Component + Send + Sync,
    {
        self.map.insert(key.into(), Box::new(PhantomData::<C>));
    }
}

#[derive(Copy, Clone)]
struct Inner<'a> {
    res: &'a Resources,
    map: &'a FnvHashMap<Cow<'static, str>, Box<DeserializeComponent>>,
}

#[derive(Copy, Clone)]
struct UiSeed<'a>(Inner<'a>);

trait DeserializeComponent {
    fn deserialize_component<'de>(
        &self,
        deserializer: &mut erased::Deserializer<'de>,
        entity: Entity,
        res: &Resources,
    ) -> Result<(), failure::Error>;
}

impl<C> DeserializeComponent for PhantomData<C>
where
    C: de::DeserializeOwned + Component + Send + Sync,
{
    fn deserialize_component<'de>(
        &self,
        deserializer: &mut erased::Deserializer<'de>,
        entity: Entity,
        res: &Resources,
    ) -> Result<(), failure::Error> {
        let comp = C::deserialize(deserializer)?;
        let mut storage = WriteStorage::<C>::fetch(res);
        storage.insert(entity, comp)?;
        Ok(())
    }
}

impl<'de, 'a> de::DeserializeSeed<'de> for UiSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<'a>(Inner<'a>);

        impl<'de, 'a> de::Visitor<'de> for Visitor<'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of entities")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                while seq.next_element_seed(EntitySeed(self.0))?.is_some() {}
                Ok(())
            }
        }

        deserializer.deserialize_seq(Visitor(self.0))
    }
}

#[derive(Copy, Clone)]
pub struct EntitySeed<'a>(Inner<'a>);

impl<'de, 'a> de::DeserializeSeed<'de> for EntitySeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<'a>(Inner<'a>);

        impl<'de, 'a> de::Visitor<'de> for Visitor<'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "map of strings to components")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let entity = self.0.res.fetch::<EntitiesRes>().create();
                while let Some(key) = map.next_key::<&str>()? {
                    if let Some(de) = self.0.map.get(key) {
                        map.next_value_seed(ComponentSeed {
                            entity,
                            res: self.0.res,
                            de: &**de,
                        })?;
                    } else {
                        return Err(<A::Error as de::Error>::custom("key not in registry"));
                    }
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor(self.0))
    }
}

#[derive(Copy, Clone)]
pub struct ComponentSeed<'a> {
    entity: Entity,
    res: &'a Resources,
    de: &'a DeserializeComponent,
}

impl<'de, 'a> de::DeserializeSeed<'de> for ComponentSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut deserializer = erased::Deserializer::erase(deserializer);
        self.de
            .deserialize_component(&mut deserializer, self.entity, self.res)
            .map_err(<D::Error as de::Error>::custom)
    }
}
