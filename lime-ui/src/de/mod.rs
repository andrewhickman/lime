mod imp;
mod registry;
#[cfg(test)]
mod tests;

pub use self::imp::deserialize;
pub use self::registry::{Deserialize, Insert, Registry};

use std::borrow::Cow;

use erased_serde as erased;
use fnv::FnvHashMap;
use specs::prelude::*;
use specs::world::EntitiesRes;

pub struct Seed<'de: 'a, 'a> {
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    res: &'a Resources,
    reg: &'a Registry,
    entity: Entity,
}

impl<'de, 'a> Seed<'de, 'a> {
    pub fn get_entity(&mut self, name: Cow<'de, str>) -> Entity {
        get_entity(name, &mut self.names, &*self.res.fetch())
    }

    pub fn deserialize(
        &mut self,
        key: Cow<'de, str>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        self.reg.get(key).and_then(|de| {
            de(
                Seed {
                    names: self.names,
                    res: self.res,
                    reg: self.reg,
                    entity: self.entity,
                },
                deserializer,
            )
        })
    }
}

fn get_entity<'de, 'a>(
    name: Cow<'de, str>,
    names: &'a mut FnvHashMap<Cow<'de, str>, Entity>,
    ents: &'a EntitiesRes,
) -> Entity {
    *names.entry(name).or_insert_with(|| ents.create())
}
