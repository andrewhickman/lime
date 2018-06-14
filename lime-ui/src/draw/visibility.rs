use std::mem;

use erased_serde as erased;
use serde::de as serde;
use shrev::EventChannel;
use specs::prelude::*;
use specs::storage::InsertResult;
use specs_mirror::{Mirrored, MirroredStorage, StorageMutExt};

use de::{DeserializeAndInsert, Seed};

#[derive(Copy, Clone, Debug)]
pub struct Visibility {
    state: VisibilityState,
}

impl Component for Visibility {
    type Storage = MirroredStorage<Self>;
}

impl Mirrored for Visibility {
    type Event = VisibilityEvent;
}

impl Visibility {
    pub fn new() -> Self {
        Visibility {
            state: VisibilityState::Visible,
        }
    }

    pub fn insert(
        entity: Entity,
        state: VisibilityState,
        storage: &mut WriteStorage<Self>,
    ) -> InsertResult<Self> {
        let res = storage.insert(entity, Visibility { state });
        if res.is_ok() && state != VisibilityState::Visible {
            storage.event_channel().single_write(VisibilityEvent {
                entity,
                old: VisibilityState::Visible,
                new: state,
            });
        }
        res
    }

    pub(crate) fn needs_draw(&self) -> bool {
        self.state == VisibilityState::Visible
    }

    pub fn get(&self) -> VisibilityState {
        self.state
    }

    pub fn set(
        &mut self,
        entity: Entity,
        new: VisibilityState,
        chan: &mut EventChannel<VisibilityEvent>,
    ) {
        let old = mem::replace(&mut self.state, new);
        if old != new {
            chan.single_write(VisibilityEvent { entity, old, new })
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename = "Visibility")]
pub enum VisibilityState {
    Visible,
    Hidden,
    Collapsed,
}

pub struct VisibilityEvent {
    pub entity: Entity,
    pub old: VisibilityState,
    pub new: VisibilityState,
}

impl VisibilityEvent {
    pub(crate) fn needs_layout_changed(&self) -> Option<bool> {
        match (self.old, self.new) {
            (VisibilityState::Collapsed, VisibilityState::Collapsed) => None,
            (VisibilityState::Collapsed, _) => Some(true),
            (_, VisibilityState::Collapsed) => Some(false),
            (_, _) => None,
        }
    }
}

impl DeserializeAndInsert for Visibility {
    fn deserialize_and_insert<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        let state = <VisibilityState as serde::Deserialize>::deserialize(deserializer)?;
        let res = Visibility::insert(seed.entity, state, &mut WriteStorage::fetch(seed.res));
        if res.unwrap().is_some() {
            Err(serde::Error::custom(format!(
                "visibility defined twice for entity '{}'",
                seed.get_name(seed.entity)
            )))
        } else {
            Ok(())
        }
    }
}
