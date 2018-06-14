mod de;
#[cfg(test)]
mod tests;

use std::any::TypeId;
use std::mem;

use shrev::EventChannel;
use specs::prelude::*;
use specs::storage::InsertResult;
use specs_mirror::{Mirrored, MirroredStorage, StorageMutExt};

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
        if !res.is_err() {
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
