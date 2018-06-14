#[cfg(test)]
mod tests;

use std::any::TypeId;
use std::borrow::Cow;
use std::mem;

use erased_serde as erased;
use serde::de as serde;
use shrev::EventChannel;
use specs::prelude::*;
use specs::storage::InsertResult;
use specs_mirror::{Mirrored, MirroredStorage, StorageMutExt};

use de::{DeserializeAndInsert, Seed};

pub struct Style {
    style: Entity,
    ty: TypeId,
}

pub struct StyleEvent {
    pub entity: Entity,
    pub style: Style,
}

impl Component for Style {
    type Storage = MirroredStorage<Self>;
}

impl Mirrored for Style {
    type Event = StyleEvent;
}

impl Style {
    pub fn insert<C: Component>(
        entity: Entity,
        style: Entity,
        storage: &mut WriteStorage<Self>,
    ) -> InsertResult<Self> {
        Self::insert_with_ty(entity, style, TypeId::of::<C>(), storage)
    }

    pub fn insert_with_ty(
        entity: Entity,
        style: Entity,
        ty: TypeId,
        storage: &mut WriteStorage<Self>,
    ) -> InsertResult<Self> {
        let res = storage.insert(entity, Style { style, ty });
        if res.is_ok() {
            storage.event_channel().single_write(StyleEvent {
                entity,
                style: Style { style, ty },
            });
        }
        res
    }

    pub fn get(&self) -> Entity {
        self.style
    }

    pub fn is<C: Component>(&self) -> bool {
        self.ty == TypeId::of::<C>()
    }

    pub fn set(&mut self, entity: Entity, style: Entity, chan: &mut EventChannel<StyleEvent>) {
        let old = mem::replace(&mut self.style, style);
        if old != style {
            chan.single_write(StyleEvent {
                entity,
                style: Style { style, ty: self.ty },
            })
        }
    }
}

impl DeserializeAndInsert for Style {
    fn deserialize_and_insert<'de, 'a>(
        mut seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        #[derive(Deserialize)]
        #[serde(rename = "Style")]
        struct StyleDe<'c> {
            #[serde(borrow)]
            style: Cow<'c, str>,
            #[serde(borrow)]
            ty: Cow<'c, str>,
        }

        let StyleDe { style, ty } = serde::Deserialize::deserialize(deserializer)?;
        let style = seed.get_entity(style);
        let ty = seed.reg.get_ty(ty)?;

        let res = Style::insert_with_ty(seed.entity, style, ty, &mut WriteStorage::fetch(seed.res));
        if res.unwrap().is_some() {
            Err(serde::Error::custom(format!(
                "style defined twice for entity '{}'",
                seed.get_name(seed.entity)
            )))
        } else {
            Ok(())
        }
    }
}
