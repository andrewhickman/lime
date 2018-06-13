use std::mem;

use shrev::EventChannel;
use specs::error::Error;
use specs::prelude::*;
use specs_mirror::{Mirrored, MirroredStorage, StorageMutExt};

pub struct Style {
    style: Entity,
}

pub struct StyleEvent {
    pub entity: Entity,
    pub style: Entity,
}

impl Component for Style {
    type Storage = MirroredStorage<Self>;
}

impl Mirrored for Style {
    type Event = StyleEvent;
}

impl Style {
    pub fn insert(
        entity: Entity,
        style: Entity,
        storage: &mut WriteStorage<Self>,
    ) -> Result<(), Error> {
        let res = storage.insert(entity, Style { style }).map(drop);
        storage
            .event_channel()
            .single_write(StyleEvent { entity, style });
        res
    }

    pub fn get(&self) -> Entity {
        self.style
    }

    pub fn set(&mut self, entity: Entity, style: Entity, chan: &mut EventChannel<StyleEvent>) {
        let old = mem::replace(&mut self.style, style);
        if old != style {
            chan.single_write(StyleEvent { entity, style })
        }
    }
}
